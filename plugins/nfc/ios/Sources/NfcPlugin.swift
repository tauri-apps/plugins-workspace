// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// https://developer.apple.com/documentation/corenfc/building_an_nfc_tag-reader_app

import CoreNFC
import SwiftRs
import Tauri
import UIKit
import WebKit

enum ScanKind: Decodable {
  case ndef, tag
}

struct ScanOptions: Decodable {
  let kind: ScanKind
  let keepSessionAlive: Bool?
  let message: String?
  let successMessage: String?
}

struct NDEFRecord: Decodable {
  let format: UInt8?
  let kind: [UInt8]?
  let identifier: [UInt8]?
  let payload: [UInt8]?
}

struct WriteOptions: Decodable {
  let kind: ScanKind?
  let records: [NDEFRecord]
  let message: String?
  let successMessage: String?
  let successfulReadMessage: String?
}

enum TagProcessMode {
  case write(message: NFCNDEFMessage)
  case read
}

class Session {
  let nfcSession: NFCReaderSession?
  let invoke: Invoke
  var keepAlive: Bool
  let tagProcessMode: TagProcessMode
  var tagStatus: NFCNDEFStatus?
  var tag: NFCNDEFTag?
  let successfulReadMessage: String?
  let successfulWriteAlertMessage: String?

  init(
    nfcSession: NFCReaderSession?,
    invoke: Invoke,
    keepAlive: Bool,
    tagProcessMode: TagProcessMode,
    successfulReadMessage: String?,
    successfulWriteAlertMessage: String?
  ) {
    self.nfcSession = nfcSession
    self.invoke = invoke
    self.keepAlive = keepAlive
    self.tagProcessMode = tagProcessMode
    self.successfulReadMessage = successfulReadMessage
    self.successfulWriteAlertMessage = successfulWriteAlertMessage
  }
}

class NfcStatus {
  let available: Bool
  let errorReason: String?

  init(available: Bool, errorReason: String?) {
    self.available = available
    self.errorReason = errorReason
  }
}

class NfcPlugin: Plugin, NFCTagReaderSessionDelegate, NFCNDEFReaderSessionDelegate {
  var session: Session?
  var status: NfcStatus!

  public override func load(webview: WKWebView) {
    var available = false
    var errorReason: String?

    let entry = Bundle.main.infoDictionary?["NFCReaderUsageDescription"] as? String

    if entry == nil || entry?.count == 0 {
      errorReason = "missing NFCReaderUsageDescription configuration on the Info.plist file"
    } else if !NFCNDEFReaderSession.readingAvailable {
      errorReason =
        "NFC tag reading unavailable, make sure the Near-Field Communication capability on Xcode is enabled and the device supports NFC tag reading"
    } else {
      available = true
    }

    if let error = errorReason {
      Logger.error("\(error)")
    }

    self.status = NfcStatus(available: available, errorReason: errorReason)
  }

  func tagReaderSessionDidBecomeActive(
    _ session: NFCTagReaderSession
  ) {
    Logger.info("tagReaderSessionDidBecomeActive")
  }

  func tagReaderSession(_ session: NFCTagReaderSession, didDetect tags: [NFCTag]) {
    let tag = tags.first!

    session.connect(
      to: tag,
      completionHandler: { [self] (error) in
        if let error = error {
          self.closeSession(session, error: "cannot connect to tag: \(error)")

        } else {
          let ndefTag: NFCNDEFTag
          switch tag {
          case let .feliCa(tag):
            ndefTag = tag as NFCNDEFTag
            break
          case let .miFare(tag):
            ndefTag = tag as NFCNDEFTag
            break
          case let .iso15693(tag):
            ndefTag = tag as NFCNDEFTag
            break
          case let .iso7816(tag):
            ndefTag = tag as NFCNDEFTag
            break
          default:
            return
          }

          self.processTag(
            session: session, tag: ndefTag, metadata: tagMetadata(tag),
            mode: self.session!.tagProcessMode)
        }
      }
    )
  }

  func tagReaderSession(_ session: NFCTagReaderSession, didInvalidateWithError error: Error) {
    Logger.error("Tag reader session error \(error)")
    self.session?.invoke.reject("session invalidated with error: \(error)")
  }

  func readerSession(_ session: NFCNDEFReaderSession, didDetectNDEFs messages: [NFCNDEFMessage]) {
    let message = messages.first!
    // TODO: do we really need this hook?
    self.session?.invoke.resolve(["records": ndefMessageRecords(message)])
  }

  func readerSession(_ session: NFCNDEFReaderSession, didDetect tags: [NFCNDEFTag]) {
    let tag = tags.first!

    session.connect(
      to: tag,
      completionHandler: { [self] (error) in
        if let error = error {
          self.closeSession(session, error: "cannot connect to tag: \(error)")

        } else {
          var metadata: JsonObject = [:]
          if tag.isKind(of: NFCFeliCaTag.self) {
            metadata["kind"] = ["FeliCa"]
            metadata["id"] = nil
          } else if let t = tag as? NFCMiFareTag {
            metadata["kind"] = ["MiFare"]
            metadata["id"] = byteArrayFromData(t.identifier)
          } else if let t = tag as? NFCISO15693Tag {
            metadata["kind"] = ["ISO15693"]
            metadata["id"] = byteArrayFromData(t.identifier)
          } else if let t = tag as? NFCISO7816Tag {
            metadata["kind"] = ["ISO7816Compatible"]
            metadata["id"] = byteArrayFromData(t.identifier)
          }

          self.processTag(
            session: session, tag: tag, metadata: metadata,
            mode: self.session!.tagProcessMode)
        }
      }
    )

  }

  func readerSession(_ session: NFCNDEFReaderSession, didInvalidateWithError error: Error) {
    if (error as NSError).code
      == NFCReaderError.Code.readerSessionInvalidationErrorFirstNDEFTagRead.rawValue
    {
      // not an error because we're using invalidateAfterFirstRead: true
      Logger.debug("readerSessionInvalidationErrorFirstNDEFTagRead")
    } else {
      Logger.error("NDEF reader session error \(error)")
      self.session?.invoke.reject("session invalidated with error: \(error)")
    }
  }

  private func tagMetadata(_ tag: NFCTag) -> JsonObject {
    var metadata: JsonObject = [:]

    switch tag {
    case .feliCa:
      metadata["kind"] = ["FeliCa"]
      metadata["id"] = []
      break
    case let .miFare(tag):
      metadata["kind"] = ["MiFare"]
      metadata["id"] = byteArrayFromData(tag.identifier)
      break
    case let .iso15693(tag):
      metadata["kind"] = ["ISO15693"]
      metadata["id"] = byteArrayFromData(tag.identifier)
      break
    case let .iso7816(tag):
      metadata["kind"] = ["ISO7816Compatible"]
      metadata["id"] = byteArrayFromData(tag.identifier)
      break
    default:
      metadata["kind"] = ["Unknown"]
      metadata["id"] = []
      break
    }

    return metadata
  }

  private func closeSession(_ session: NFCReaderSession) {
    session.invalidate()
    self.session = nil
  }

  private func closeSession(_ session: NFCReaderSession, error: String) {
    session.invalidate(errorMessage: error)
    self.session = nil
  }

  private func processTag<T: NFCNDEFTag>(
    session: NFCReaderSession, tag: T, metadata: JsonObject, mode: TagProcessMode
  ) {
    tag.queryNDEFStatus(completionHandler: {
      [self] (status, capacity, error) in
      if let error = error {
        self.closeSession(session, error: "cannot connect to tag: \(error)")
      } else {
        switch mode {
        case .write(let message):
          self.writeNDEFTag(
            session: session, status: status, tag: tag, message: message,
            alertMessage: self.session?.successfulWriteAlertMessage)
          break
        case .read:
          if self.session?.keepAlive == true {
            self.session!.tagStatus = status
            self.session!.tag = tag
          }
          self.readNDEFTag(
            session: session, status: status, tag: tag, metadata: metadata,
            alertMessage: self.session?.successfulReadMessage)
          break
        }
      }
    })
  }

  private func writeNDEFTag<T: NFCNDEFTag>(
    session: NFCReaderSession, status: NFCNDEFStatus, tag: T, message: NFCNDEFMessage,
    alertMessage: String?
  ) {
    switch status {
    case .notSupported:
      self.closeSession(session, error: "Tag is not an NDEF-formatted tag")
      break
    case .readOnly:
      self.closeSession(session, error: "Read only tag")
      break
    case .readWrite:
      if let currentSession = self.session {
        tag.writeNDEF(
          message,
          completionHandler: { (error) in
            if let error = error {
              self.closeSession(session, error: "cannot write to tag: \(error)")
            } else {
              if let message = alertMessage {
                session.alertMessage = message
              }
              currentSession.invoke.resolve()

              self.closeSession(session)

            }
          })
      }
      break
    default:
      return
    }
  }

  private func readNDEFTag<T: NFCNDEFTag>(
    session: NFCReaderSession, status: NFCNDEFStatus, tag: T, metadata m: JsonObject,
    alertMessage: String?
  ) {
    var metadata: JsonObject = [:]
    metadata.merge(m) { (_, new) in new }

    switch status {
    case .notSupported:
      self.resolveInvoke(message: nil, metadata: metadata)
      self.closeSession(session)
      return
    case .readOnly:
      metadata["readOnly"] = true
      break
    case .readWrite:
      metadata["readOnly"] = false
      break
    default:
      break
    }

    tag.readNDEF(completionHandler: {
      [self] (message, error) in
      if let error = error {
        let code = (error as NSError).code
        if code != 403 {
          self.closeSession(session, error: "Failed to read: \(error)")
          return
        }
      }

      if let message = alertMessage {
        session.alertMessage = message
      }
      self.resolveInvoke(message: message, metadata: metadata)

      if self.session?.keepAlive != true {
        self.closeSession(session)
      }
    })
  }

  private func resolveInvoke(message: NFCNDEFMessage?, metadata: JsonObject) {
    var data: JsonObject = [:]

    data.merge(metadata) { (_, new) in new }

    if let message = message {
      data["records"] = ndefMessageRecords(message)
    } else {
      data["records"] = []
    }

    self.session?.invoke.resolve(data)
  }

  private func ndefMessageRecords(_ message: NFCNDEFMessage) -> [JsonObject] {
    var records: [JsonObject] = []
    for record in message.records {
      var recordJson: JsonObject = [:]
      recordJson["tnf"] = record.typeNameFormat.rawValue
      recordJson["kind"] = byteArrayFromData(record.type)
      recordJson["id"] = byteArrayFromData(record.identifier)
      recordJson["payload"] = byteArrayFromData(record.payload)

      records.append(recordJson)
    }

    return records
  }

  private func byteArrayFromData(_ data: Data) -> [UInt8] {
    var arr: [UInt8] = []
    for b in data {
      arr.append(b)
    }
    return arr
  }

  private func dataFromByteArray(_ array: [UInt8]) -> Data {
    var data = Data(capacity: array.count)

    data.append(contentsOf: array)

    return data
  }

  @objc func isAvailable(_ invoke: Invoke) {
    invoke.resolve([
      "available": self.status.available
    ])
  }

  @objc public func write(_ invoke: Invoke) throws {
    if !self.status.available {
      invoke.reject("NFC reading unavailable: \(self.status.errorReason ?? "")")
      return
    }

    let args = try invoke.parseArgs(WriteOptions.self)

    var ndefPayloads = [NFCNDEFPayload]()

    for record in args.records {
      ndefPayloads.append(
        NFCNDEFPayload(
          format: NFCTypeNameFormat(rawValue: record.format ?? 0) ?? .unknown,
          type: dataFromByteArray(record.kind ?? []),
          identifier: dataFromByteArray(record.identifier ?? []),
          payload: dataFromByteArray(record.payload ?? [])
        )
      )
    }

    if let session = self.session {
      if let nfcSession = session.nfcSession, let tagStatus = session.tagStatus,
        let tag = session.tag
      {
        session.keepAlive = false
        self.writeNDEFTag(
          session: nfcSession, status: tagStatus, tag: tag,
          message: NFCNDEFMessage(records: ndefPayloads),
          alertMessage: args.successMessage
        )
      } else {
        invoke.reject(
          "connected tag not found, please wait for it to be available and then call write()")
      }
    } else {
      self.startScanSession(
        invoke: invoke,
        kind: args.kind ?? .ndef,
        keepAlive: true,
        invalidateAfterFirstRead: false,
        tagProcessMode: .write(
          message: NFCNDEFMessage(records: ndefPayloads)
        ),
        alertMessage: args.message,
        successfulReadMessage: args.successfulReadMessage,
        successfulWriteAlertMessage: args.successMessage
      )
    }
  }

  @objc public func scan(_ invoke: Invoke) throws {
    if !self.status.available {
      invoke.reject("NFC reading unavailable: \(self.status.errorReason ?? "")")
      return
    }

    let args = try invoke.parseArgs(ScanOptions.self)

    self.startScanSession(
      invoke: invoke,
      kind: args.kind,
      keepAlive: args.keepSessionAlive ?? false,
      invalidateAfterFirstRead: true,
      tagProcessMode: .read,
      alertMessage: args.message,
      successfulReadMessage: args.successMessage,
      successfulWriteAlertMessage: nil
    )
  }

  private func startScanSession(
    invoke: Invoke,
    kind: ScanKind,
    keepAlive: Bool,
    invalidateAfterFirstRead: Bool,
    tagProcessMode: TagProcessMode,
    alertMessage: String?,
    successfulReadMessage: String?,
    successfulWriteAlertMessage: String?
  ) {
    let nfcSession: NFCReaderSession?

    switch kind {
    case .tag:
      nfcSession = NFCTagReaderSession(
        pollingOption: [.iso14443, .iso15693],
        delegate: self,
        queue: DispatchQueue.main
      )
      break
    case .ndef:
      nfcSession = NFCNDEFReaderSession(
        delegate: self,
        queue: DispatchQueue.main,
        invalidateAfterFirstRead: invalidateAfterFirstRead
      )
      break
    }

    if let message = alertMessage {
      nfcSession?.alertMessage = message
    }
    nfcSession?.begin()

    self.session = Session(
      nfcSession: nfcSession,
      invoke: invoke,
      keepAlive: keepAlive,
      tagProcessMode: tagProcessMode,
      successfulReadMessage: successfulReadMessage,
      successfulWriteAlertMessage: successfulWriteAlertMessage
    )
  }
}

@_cdecl("init_plugin_nfc")
func initPlugin() -> Plugin {
  return NfcPlugin()
}
