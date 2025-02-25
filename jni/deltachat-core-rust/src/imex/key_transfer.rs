//! # Key transfer via Autocrypt Setup Message.
use rand::{thread_rng, Rng};

use anyhow::{bail, ensure, Result};

use crate::blob::BlobObject;
use crate::chat::{self, ChatId};
use crate::config::Config;
use crate::contact::ContactId;
use crate::context::Context;
use crate::imex::maybe_add_bcc_self_device_msg;
use crate::imex::set_self_key;
use crate::key::{load_self_secret_key, DcKey};
use crate::message::{Message, MsgId, Viewtype};
use crate::mimeparser::SystemMessage;
use crate::param::Param;
use crate::pgp;
use crate::stock_str;
use crate::tools::open_file_std;

/// Initiates key transfer via Autocrypt Setup Message.
///
/// Returns setup code.
pub async fn initiate_key_transfer(context: &Context) -> Result<String> {
    let setup_code = create_setup_code(context);
    /* this may require a keypair to be created. this may take a second ... */
    let setup_file_content = render_setup_file(context, &setup_code).await?;
    /* encrypting may also take a while ... */
    let setup_file_blob = BlobObject::create(
        context,
        "autocrypt-setup-message.html",
        setup_file_content.as_bytes(),
    )
    .await?;

    let chat_id = ChatId::create_for_contact(context, ContactId::SELF).await?;
    let mut msg = Message {
        viewtype: Viewtype::File,
        ..Default::default()
    };
    msg.param.set(Param::File, setup_file_blob.as_name());
    msg.subject = stock_str::ac_setup_msg_subject(context).await;
    msg.param
        .set(Param::MimeType, "application/autocrypt-setup");
    msg.param.set_cmd(SystemMessage::AutocryptSetupMessage);
    msg.force_plaintext();
    msg.param.set_int(Param::SkipAutocrypt, 1);

    chat::send_msg(context, chat_id, &mut msg).await?;
    // no maybe_add_bcc_self_device_msg() here.
    // the ui shows the dialog with the setup code on this device,
    // it would be too much noise to have two things popping up at the same time.
    // maybe_add_bcc_self_device_msg() is called on the other device
    // once the transfer is completed.
    Ok(setup_code)
}

/// Continue key transfer via Autocrypt Setup Message.
///
/// `msg_id` is the ID of the received Autocrypt Setup Message.
/// `setup_code` is the code entered by the user.
pub async fn continue_key_transfer(
    context: &Context,
    msg_id: MsgId,
    setup_code: &str,
) -> Result<()> {
    ensure!(!msg_id.is_special(), "wrong id");

    let msg = Message::load_from_db(context, msg_id).await?;
    ensure!(
        msg.is_setupmessage(),
        "Message is no Autocrypt Setup Message."
    );

    if let Some(filename) = msg.get_file(context) {
        let file = open_file_std(context, filename)?;
        let sc = normalize_setup_code(setup_code);
        let armored_key = decrypt_setup_file(&sc, file).await?;
        set_self_key(context, &armored_key, true).await?;
        maybe_add_bcc_self_device_msg(context).await?;

        Ok(())
    } else {
        bail!("Message is no Autocrypt Setup Message.");
    }
}

/// Renders HTML body of a setup file message.
///
/// The `passphrase` must be at least 2 characters long.
pub async fn render_setup_file(context: &Context, passphrase: &str) -> Result<String> {
    let passphrase_begin = if let Some(passphrase_begin) = passphrase.get(..2) {
        passphrase_begin
    } else {
        bail!("Passphrase must be at least 2 chars long.");
    };
    let private_key = load_self_secret_key(context).await?;
    let ac_headers = match context.get_config_bool(Config::E2eeEnabled).await? {
        false => None,
        true => Some(("Autocrypt-Prefer-Encrypt", "mutual")),
    };
    let private_key_asc = private_key.to_asc(ac_headers);
    let encr = pgp::symm_encrypt(passphrase, private_key_asc.as_bytes())
        .await?
        .replace('\n', "\r\n");

    let replacement = format!(
        concat!(
            "-----BEGIN PGP MESSAGE-----\r\n",
            "Passphrase-Format: numeric9x4\r\n",
            "Passphrase-Begin: {}"
        ),
        passphrase_begin
    );
    let pgp_msg = encr.replace("-----BEGIN PGP MESSAGE-----", &replacement);

    let msg_subj = stock_str::ac_setup_msg_subject(context).await;
    let msg_body = stock_str::ac_setup_msg_body(context).await;
    let msg_body_html = msg_body.replace('\r', "").replace('\n', "<br>");
    Ok(format!(
        concat!(
            "<!DOCTYPE html>\r\n",
            "<html>\r\n",
            "  <head>\r\n",
            "    <title>{}</title>\r\n",
            "  </head>\r\n",
            "  <body>\r\n",
            "    <h1>{}</h1>\r\n",
            "    <p>{}</p>\r\n",
            "    <pre>\r\n{}\r\n</pre>\r\n",
            "  </body>\r\n",
            "</html>\r\n"
        ),
        msg_subj, msg_subj, msg_body_html, pgp_msg
    ))
}

/// Creates a new setup code for Autocrypt Setup Message.
fn create_setup_code(_context: &Context) -> String {
    let mut random_val: u16;
    let mut rng = thread_rng();
    let mut ret = String::new();

    for i in 0..9 {
        loop {
            random_val = rng.gen();
            if random_val as usize <= 60000 {
                break;
            }
        }
        random_val = (random_val as usize % 10000) as u16;
        ret += &format!(
            "{}{:04}",
            if 0 != i { "-" } else { "" },
            random_val as usize
        );
    }

    ret
}

async fn decrypt_setup_file<T: std::io::Read + std::io::Seek>(
    passphrase: &str,
    file: T,
) -> Result<String> {
    let plain_bytes = pgp::symm_decrypt(passphrase, file).await?;
    let plain_text = std::string::String::from_utf8(plain_bytes)?;

    Ok(plain_text)
}

fn normalize_setup_code(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            out.push(c);
            if let 4 | 9 | 14 | 19 | 24 | 29 | 34 | 39 = out.len() {
                out += "-"
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::pgp::{split_armored_data, HEADER_AUTOCRYPT, HEADER_SETUPCODE};
    use crate::receive_imf::receive_imf;
    use crate::stock_str::StockMessage;
    use crate::test_utils::{TestContext, TestContextManager};
    use ::pgp::armor::BlockType;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_render_setup_file() {
        let t = TestContext::new_alice().await;
        let msg = render_setup_file(&t, "hello").await.unwrap();
        println!("{}", &msg);
        // Check some substrings, indicating things got substituted.
        assert!(msg.contains("<title>Autocrypt Setup Message</title"));
        assert!(msg.contains("<h1>Autocrypt Setup Message</h1>"));
        assert!(msg.contains("<p>This is the Autocrypt Setup Message used to"));
        assert!(msg.contains("-----BEGIN PGP MESSAGE-----\r\n"));
        assert!(msg.contains("Passphrase-Format: numeric9x4\r\n"));
        assert!(msg.contains("Passphrase-Begin: he\r\n"));
        assert!(msg.contains("-----END PGP MESSAGE-----\r\n"));

        for line in msg.rsplit_terminator('\n') {
            assert!(line.ends_with('\r'));
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_render_setup_file_newline_replace() {
        let t = TestContext::new_alice().await;
        t.set_stock_translation(StockMessage::AcSetupMsgBody, "hello\r\nthere".to_string())
            .await
            .unwrap();
        let msg = render_setup_file(&t, "pw").await.unwrap();
        println!("{}", &msg);
        assert!(msg.contains("<p>hello<br>there</p>"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_create_setup_code() {
        let t = TestContext::new().await;
        let setupcode = create_setup_code(&t);
        assert_eq!(setupcode.len(), 44);
        assert_eq!(setupcode.chars().nth(4).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(9).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(14).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(19).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(24).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(29).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(34).unwrap(), '-');
        assert_eq!(setupcode.chars().nth(39).unwrap(), '-');
    }

    #[test]
    fn test_normalize_setup_code() {
        let norm = normalize_setup_code("123422343234423452346234723482349234");
        assert_eq!(norm, "1234-2234-3234-4234-5234-6234-7234-8234-9234");

        let norm =
            normalize_setup_code("\t1 2 3422343234- foo bar-- 423-45 2 34 6234723482349234      ");
        assert_eq!(norm, "1234-2234-3234-4234-5234-6234-7234-8234-9234");
    }

    /* S_EM_SETUPFILE is a AES-256 symm. encrypted setup message created by Enigmail
    with an "encrypted session key", see RFC 4880.  The code is in S_EM_SETUPCODE */
    const S_EM_SETUPCODE: &str = "1742-0185-6197-1303-7016-8412-3581-4441-0597";
    const S_EM_SETUPFILE: &str = include_str!("../../test-data/message/stress.txt");

    // Autocrypt Setup Message payload "encrypted" with plaintext algorithm.
    const S_PLAINTEXT_SETUPFILE: &str =
        include_str!("../../test-data/message/plaintext-autocrypt-setup.txt");

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_split_and_decrypt() {
        let buf_1 = S_EM_SETUPFILE.as_bytes().to_vec();
        let (typ, headers, base64) = split_armored_data(&buf_1).unwrap();
        assert_eq!(typ, BlockType::Message);
        assert!(S_EM_SETUPCODE.starts_with(headers.get(HEADER_SETUPCODE).unwrap()));
        assert!(!headers.contains_key(HEADER_AUTOCRYPT));

        assert!(!base64.is_empty());

        let setup_file = S_EM_SETUPFILE.to_string();
        let decrypted =
            decrypt_setup_file(S_EM_SETUPCODE, std::io::Cursor::new(setup_file.as_bytes()))
                .await
                .unwrap();

        let (typ, headers, _base64) = split_armored_data(decrypted.as_bytes()).unwrap();

        assert_eq!(typ, BlockType::PrivateKey);
        assert_eq!(headers.get(HEADER_AUTOCRYPT), Some(&"mutual".to_string()));
        assert!(!headers.contains_key(HEADER_SETUPCODE));
    }

    /// Tests that Autocrypt Setup Message encrypted with "plaintext" algorithm cannot be
    /// decrypted.
    ///
    /// According to <https://datatracker.ietf.org/doc/html/rfc4880#section-13.4>
    /// "Implementations MUST NOT use plaintext in Symmetrically Encrypted Data packets".
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_decrypt_plaintext_autocrypt_setup_message() {
        let setup_file = S_PLAINTEXT_SETUPFILE.to_string();
        let incorrect_setupcode = "0000-0000-0000-0000-0000-0000-0000-0000-0000";
        assert!(decrypt_setup_file(
            incorrect_setupcode,
            std::io::Cursor::new(setup_file.as_bytes()),
        )
        .await
        .is_err());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_key_transfer() -> Result<()> {
        let alice = TestContext::new_alice().await;

        let setup_code = initiate_key_transfer(&alice).await?;

        // Get Autocrypt Setup Message.
        let sent = alice.pop_sent_msg().await;

        // Alice sets up a second device.
        let alice2 = TestContext::new().await;
        alice2.set_name("alice2");
        alice2.configure_addr("alice@example.org").await;
        alice2.recv_msg(&sent).await;
        let msg = alice2.get_last_msg().await;
        assert!(msg.is_setupmessage());

        // Send a message that cannot be decrypted because the keys are
        // not synchronized yet.
        let sent = alice2.send_text(msg.chat_id, "Test").await;
        let trashed_message = alice.recv_msg_opt(&sent).await;
        assert!(trashed_message.is_none());
        assert_ne!(alice.get_last_msg().await.get_text(), "Test");

        // Transfer the key.
        continue_key_transfer(&alice2, msg.id, &setup_code).await?;

        // Alice sends a message to self from the new device.
        let sent = alice2.send_text(msg.chat_id, "Test").await;
        alice.recv_msg(&sent).await;
        assert_eq!(alice.get_last_msg().await.get_text(), "Test");

        Ok(())
    }

    /// Tests that Autocrypt Setup Messages is only clickable if it is self-sent.
    /// This prevents Bob from tricking Alice into changing the key
    /// by sending her an Autocrypt Setup Message as long as Alice's server
    /// does not allow to forge the `From:` header.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_key_transfer_non_self_sent() -> Result<()> {
        let mut tcm = TestContextManager::new();
        let alice = tcm.alice().await;
        let bob = tcm.bob().await;

        let _setup_code = initiate_key_transfer(&alice).await?;

        // Get Autocrypt Setup Message.
        let sent = alice.pop_sent_msg().await;

        let rcvd = bob.recv_msg(&sent).await;
        assert!(!rcvd.is_setupmessage());

        Ok(())
    }

    /// Tests reception of Autocrypt Setup Message from K-9 6.802.
    ///
    /// Unlike Autocrypt Setup Message sent by Delta Chat,
    /// this message does not contain `Autocrypt-Prefer-Encrypt` header.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_key_transfer_k_9() -> Result<()> {
        let t = &TestContext::new().await;
        t.configure_addr("autocrypt@nine.testrun.org").await;

        let raw = include_bytes!("../../test-data/message/k-9-autocrypt-setup-message.eml");
        let received = receive_imf(t, raw, false).await?.unwrap();

        let setup_code = "0655-9868-8252-5455-4232-5158-1237-5333-2638";
        continue_key_transfer(t, *received.msg_ids.last().unwrap(), setup_code).await?;

        Ok(())
    }
}
