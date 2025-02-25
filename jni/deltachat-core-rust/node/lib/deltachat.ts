/* eslint-disable camelcase */

import binding from './binding'
import { EventId2EventName } from './constants'
import { EventEmitter } from 'events'
import { existsSync } from 'fs'
import rawDebug from 'debug'
import { tmpdir } from 'os'
import { join } from 'path'
import { Context } from './context'
const debug = rawDebug('deltachat:node:index')

const noop = function () {}
interface NativeAccount {}

/**
 * Wrapper around dcn_account_t*
 */
export class AccountManager extends EventEmitter {
  dcn_accounts: NativeAccount
  accountDir: string
  jsonRpcStarted = false

  constructor(cwd: string, writable = true) {
    super()
    debug('DeltaChat constructor')

    this.accountDir = cwd
    this.dcn_accounts = binding.dcn_accounts_new(
      this.accountDir,
      writable ? 1 : 0
    )
  }

  getAllAccountIds() {
    return binding.dcn_accounts_get_all(this.dcn_accounts)
  }

  selectAccount(account_id: number) {
    return binding.dcn_accounts_select_account(this.dcn_accounts, account_id)
  }

  selectedAccount(): number {
    return binding.dcn_accounts_get_selected_account(this.dcn_accounts)
  }

  addAccount(): number {
    return binding.dcn_accounts_add_account(this.dcn_accounts)
  }

  addClosedAccount(): number {
    return binding.dcn_accounts_add_closed_account(this.dcn_accounts)
  }

  removeAccount(account_id: number) {
    return binding.dcn_accounts_remove_account(this.dcn_accounts, account_id)
  }

  accountContext(account_id: number) {
    const native_context = binding.dcn_accounts_get_account(
      this.dcn_accounts,
      account_id
    )
    if (native_context === null) {
      throw new Error(
        `could not get context with id ${account_id}, does it even exist? please check your ids`
      )
    }
    return new Context(this, native_context, account_id)
  }

  migrateAccount(dbfile: string): number {
    return binding.dcn_accounts_migrate_account(this.dcn_accounts, dbfile)
  }

  close() {
    this.stopIO()
    debug('unrefing context')
    binding.dcn_accounts_unref(this.dcn_accounts)
    debug('Unref end')
  }

  emit(
    event: string | symbol,
    account_id: number,
    data1: any,
    data2: any
  ): boolean {
    super.emit('ALL', event, account_id, data1, data2)
    return super.emit(event, account_id, data1, data2)
  }

  handleCoreEvent(
    eventId: number,
    accountId: number,
    data1: number,
    data2: number | string
  ) {
    const eventString = EventId2EventName[eventId]
    debug('event', eventString, accountId, data1, data2)
    debug(eventString, data1, data2)
    if (!this.emit) {
      console.log('Received an event but EventEmitter is already destroyed.')
      console.log(eventString, data1, data2)
      return
    }
    this.emit(eventString, accountId, data1, data2)
  }

  startEvents() {
    if (this.dcn_accounts === null) {
      throw new Error('dcn_account is null')
    }
    binding.dcn_accounts_start_event_handler(
      this.dcn_accounts,
      this.handleCoreEvent.bind(this)
    )
    debug('Started event handler')
  }

  startJsonRpcHandler(callback: ((response: string) => void) | null) {
    if (this.dcn_accounts === null) {
      throw new Error('dcn_account is null')
    }
    if (!callback) {
      throw new Error('no callback set')
    }
    if (this.jsonRpcStarted) {
      throw new Error('jsonrpc was started already')
    }

    binding.dcn_accounts_start_jsonrpc(this.dcn_accounts, callback.bind(this))
    debug('Started JSON-RPC handler')
    this.jsonRpcStarted = true
  }

  jsonRpcRequest(message: string) {
    if (!this.jsonRpcStarted) {
      throw new Error(
        'jsonrpc is not active, start it with startJsonRpcHandler first'
      )
    }
    binding.dcn_json_rpc_request(this.dcn_accounts, message)
  }

  startIO() {
    binding.dcn_accounts_start_io(this.dcn_accounts)
  }

  stopIO() {
    binding.dcn_accounts_stop_io(this.dcn_accounts)
  }

  static maybeValidAddr(addr: string) {
    debug('DeltaChat.maybeValidAddr')
    if (addr === null) return false
    return Boolean(binding.dcn_maybe_valid_addr(addr))
  }

  static parseGetInfo(info: string) {
    debug('static _getInfo')
    const result: { [key: string]: string } = {}

    const regex = /^(\w+)=(.*)$/i
    info
      .split('\n')
      .filter(Boolean)
      .forEach((line) => {
        const match = regex.exec(line)
        if (match) {
          result[match[1]] = match[2]
        }
      })

    return result
  }

  static newTemporary() {
    let directory = null
    while (true) {
      const randomString = Math.random().toString(36).substring(2, 5)
      directory = join(tmpdir(), 'deltachat-' + randomString)
      if (!existsSync(directory)) break
    }
    const dc = new AccountManager(directory)
    const accountId = dc.addAccount()
    const context = dc.accountContext(accountId)
    return { dc, context, accountId, directory }
  }

  static getSystemInfo() {
    debug('DeltaChat.getSystemInfo')
    const { dc, context } = AccountManager.newTemporary()
    const info = AccountManager.parseGetInfo(
      binding.dcn_get_info(context.dcn_context)
    )
    const {
      deltachat_core_version,
      sqlite_version,
      sqlite_thread_safe,
      libetpan_version,
      openssl_version,
      compile_date,
      arch,
    } = info
    const result = {
      deltachat_core_version,
      sqlite_version,
      sqlite_thread_safe,
      libetpan_version,
      openssl_version,
      compile_date,
      arch,
    }
    context.unref()
    dc.close()
    return result
  }

  /** get information about the provider
   *
   * This function creates a temporary context to be standalone,
   * if possible use `Context.getProviderFromEmail` instead. (otherwise potential proxy settings are not used)
   * @deprecated
   */
  static getProviderFromEmail(email: string) {
    debug('DeltaChat.getProviderFromEmail')
    const { dc, context } = AccountManager.newTemporary()
    const provider = context.getProviderFromEmail(email)
    context.unref()
    dc.close()
    return provider
  }
}
