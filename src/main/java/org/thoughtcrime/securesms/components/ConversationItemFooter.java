package org.thoughtcrime.securesms.components;

import android.content.Context;
import android.content.res.TypedArray;
import android.graphics.Color;
import android.util.AttributeSet;
import android.view.View;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;

import com.b44t.messenger.DcMsg;
import com.b44t.messenger.DcContact;
import com.b44t.messenger.PrivJNI;

import org.thoughtcrime.securesms.R;
import org.thoughtcrime.securesms.util.DateUtils;

public class ConversationItemFooter extends LinearLayout {

  private TextView            dateView;
  private ImageView           secureIndicatorView,securePrvIndicatorView;
  private ImageView           locationIndicatorView;
  private View                view_file_state_indicator;
  private DeliveryStatusView  deliveryStatusView;
  private Integer             textColor = null;
  private PrivJNI             privJNI = null;
  public ConversationItemFooter(Context context) {
    super(context);
    privJNI = new PrivJNI(context);
    init(null);
  }

  public ConversationItemFooter(Context context, @Nullable AttributeSet attrs) {
    super(context, attrs);
    privJNI = new PrivJNI(context);
    init(attrs);
  }

  public ConversationItemFooter(Context context, @Nullable AttributeSet attrs, int defStyleAttr) {
    super(context, attrs, defStyleAttr);
    privJNI = new PrivJNI(context);
    init(attrs);
  }

  private void init(@Nullable AttributeSet attrs) {
    inflate(getContext(), R.layout.conversation_item_footer, this);

    dateView              = findViewById(R.id.footer_date);
    secureIndicatorView   = findViewById(R.id.footer_secure_indicator);
    securePrvIndicatorView   = findViewById(R.id.footer_prv_indicator);
    view_file_state_indicator   = findViewById(R.id.view_file_state_indicator);
    locationIndicatorView = findViewById(R.id.footer_location_indicator);
    deliveryStatusView    = new DeliveryStatusView(findViewById(R.id.delivery_indicator));

    if (attrs != null) {
      TypedArray typedArray = getContext().getTheme().obtainStyledAttributes(attrs, R.styleable.ConversationItemFooter, 0, 0);
      textColor = typedArray.getInt(R.styleable.ConversationItemFooter_footer_text_color, getResources().getColor(R.color.core_white));
      setTextColor(textColor);
      typedArray.recycle();
    }
  }

  public void setMessageRecord(@NonNull DcMsg messageRecord) {
    presentDate(messageRecord);
    if(messageRecord.isSecure()) {
      secureIndicatorView.setVisibility(VISIBLE);
      if (privJNI.isChatPrivittyProtected(messageRecord.getChatId()) && (messageRecord.getType() == DcMsg.DC_MSG_FILE))
      {
        securePrvIndicatorView.setVisibility(VISIBLE);
        view_file_state_indicator.setVisibility(VISIBLE);

        if (messageRecord.getFromId() != DcContact.DC_CONTACT_ID_SELF) {
          int fileState = privJNI.getFileAccessState(messageRecord.getChatId(), messageRecord.getFilename());
          if (fileState == PrivJNI.PRV_SSS_STATE_TYPE_SSS_ACTIVE) {
            // access allowed
            view_file_state_indicator.setBackgroundResource(R.drawable.prv_file_indicator_background_green);
          } else if (fileState == PrivJNI.PRV_SSS_STATE_TYPE_SSS_REQUEST) {
            // access requested
            view_file_state_indicator.setBackgroundResource(R.drawable.prv_file_indicator_background_blue);
          } else if ((fileState == PrivJNI.PRV_SSS_STATE_TYPE_SSS_BLOCKED) || (fileState == PrivJNI.PRV_SSS_STATE_TYPE_SSS_REVOKED)) {
            // access blocked or expired or Revoked
            view_file_state_indicator.setBackgroundResource(R.drawable.prv_file_indicator_background_red);
          }
        } else {
          view_file_state_indicator.setBackgroundResource(R.drawable.prv_file_indicator_background_green);
        }
      }
      else
      {
        securePrvIndicatorView.setVisibility(GONE);
        view_file_state_indicator.setVisibility(GONE);
      }
    } else {
      secureIndicatorView.setVisibility(GONE);
    }
    locationIndicatorView.setVisibility(messageRecord.hasLocation() ? View.VISIBLE : View.GONE);
    presentDeliveryStatus(messageRecord);
  }

  private void setTextColor(int color) {
    dateView.setTextColor(color);
    secureIndicatorView.setColorFilter(color);
    locationIndicatorView.setColorFilter(color);
    deliveryStatusView.setTint(color);
  }

  private void presentDate(@NonNull DcMsg messageRecord) {
    dateView.forceLayout();
    dateView.setText(DateUtils.getExtendedRelativeTimeSpanString(getContext(), messageRecord.getTimestamp()));
  }

  private void presentDeliveryStatus(@NonNull DcMsg messageRecord) {
    // isDownloading is temporary and should be checked first.
    boolean isDownloading = messageRecord.getDownloadState() == DcMsg.DC_DOWNLOAD_IN_PROGRESS;

         if (isDownloading)                deliveryStatusView.setDownloading();
    else if (messageRecord.isFailed())     deliveryStatusView.setFailed();
    else if (!messageRecord.isOutgoing())  deliveryStatusView.setNone();
    else if (messageRecord.isRemoteRead()) deliveryStatusView.setRead();
    else if (messageRecord.isDelivered())  deliveryStatusView.setSent();
    else if (messageRecord.isPreparing())  deliveryStatusView.setPreparing();
    else                                   deliveryStatusView.setPending();

    if (messageRecord.isFailed()) {
      deliveryStatusView.setTint(Color.RED);
    } else {
      deliveryStatusView.setTint(textColor); // Reset the color to the standard color (because the footer is re-used in a RecyclerView)
    }
  }

  public String getDescription() {
      String desc = dateView.getText().toString();
      String deliveryDesc = deliveryStatusView.getDescription();
      if (!"".equals(deliveryDesc)) {
          desc += "\n" + deliveryDesc;
      }
      return desc;
  }
}
