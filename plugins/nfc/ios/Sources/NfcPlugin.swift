// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import CoreNFC
import SwiftRs
import Tauri
import UIKit
import WebKit

enum ScanKind {
  case ndef, tag
}

enum TagProcessMode {
  case write, read
}

class Session {
  let nfcSession: NFCReaderSession?
  let invoke: Invoke
  let keepAlive: Bool
  let tagProcessMode: TagProcessMode
  var tagStatus: NFCNDEFStatus?
  var tag: NFCNDEFTag?
  var writeMessage: NFCNDEFMessage?

  init(
    nfcSession: NFCReaderSession?, invoke: Invoke, keepAlive: Bool, tagProcessMode: TagProcessMode
  ) {
    self.nfcSession = nfcSession
    self.invoke = invoke
    self.keepAlive = keepAlive
    self.tagProcessMode = tagProcessMode
  }
}

class NfcPlugin: Plugin, NFCTagReaderSessionDelegate, NFCNDEFReaderSessionDelegate {
  var session: Session?

  func tagReaderSessionDidBecomeActive(
    _ session: NFCTagReaderSession
  ) {
    Logger.info("tagReaderSessionDidBecomeActive")
  }

  func tagReaderSession(_ session: NFCTagReaderSession, didDetect tags: [NFCTag]) {
    Logger.info("detected tags!")
    let tag = tags.first!

    session.connect(
      to: tag,
      completionHandler: { [self] (error) in
        if let error = error {
          if self.session?.keepAlive != true {
            self.closeSession(session, error: "cannot connect to tag: \(error)")
          }
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
    Logger.error("Tag reader session error")
    self.session?.invoke.reject("session invalidated with error: \(error)")
  }

  func readerSession(_ session: NFCNDEFReaderSession, didDetectNDEFs messages: [NFCNDEFMessage]) {
    var jsonMessages: [JsonObject] = []
    for message in messages {
      jsonMessages.append(ndefMessageToJson(message))
    }
    self.session?.invoke.resolve(["messages": jsonMessages])
  }

  func readerSession(_ session: NFCNDEFReaderSession, didDetect tags: [NFCNDEFTag]) {
    let tag = tags.first!

    session.connect(
      to: tag,
      completionHandler: { [self] (error) in
        if let error = error {
          if self.session?.keepAlive != true {
            self.closeSession(session, error: "cannot connect to tag: \(error)")
          }
        } else {
          self.processTag(
            session: session, tag: tag, metadata: [:],
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
      Logger.error("NDEF reader session error")
      self.session?.invoke.reject("session invalidated with error: \(error)")
    }
  }

  private func tagMetadata(_ tag: NFCTag) -> JsonObject {
    var metadata: JsonObject = [:]

    switch tag {
    case .feliCa:
      metadata["type"] = "FeliCa"
      metadata["id"] = nil
      break
    case let .miFare(tag):
      metadata["type"] = "MiFare"
      metadata["id"] = tag.identifier
      break
    case let .iso15693(tag):
      metadata["type"] = "ISO15693"
      metadata["id"] = tag.identifier
      break
    case let .iso7816(tag):
      metadata["type"] = "ISO7816Compatible"
      metadata["id"] = tag.identifier
      break
    default:
      metadata["type"] = "Unknown"
      metadata["id"] = nil
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
        if self.session?.keepAlive != true {
          self.closeSession(session, error: "cannot connect to tag: \(error)")
        }
      } else {
        switch mode {
        case .write:
          self.writeNDEFTag(session: session, status: status, tag: tag)
          break
        case .read:
          if self.session?.keepAlive == true {
            self.session!.tagStatus = status
            self.session!.tag = tag
          }
          self.readNDEFTag(session: session, status: status, tag: tag, metadata: metadata)
          break
        }
      }
    })
  }

  private func writeNDEFTag<T: NFCNDEFTag>(session: NFCReaderSession, status: NFCNDEFStatus, tag: T)
  {
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
          currentSession.writeMessage!,
          completionHandler: { (error) in
            if let error = error {
              if currentSession.keepAlive != true {
                self.closeSession(session, error: "cannot write to tag: \(error)")
              }
            } else {
              session.alertMessage = "Data wrote to NFC tag"
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
    session: NFCReaderSession, status: NFCNDEFStatus, tag: T, metadata: JsonObject
  ) {
    tag.readNDEF(completionHandler: {
      [self] (message, error) in
      if let error = error {
        let code = (error as NSError).code
        if code != 403 {
          self.closeSession(session, error: "Failed to read: \(error)")
          return
        }
      }

      session.alertMessage = "Successfully read tag"
      self.resolveInvoke(message: message, metadata: metadata)
      self.closeSession(session)
    })
  }

  private func resolveInvoke(message: NFCNDEFMessage?, metadata: JsonObject) {
    var data: JsonObject = [:]

    if let message = message {
      var tag = ndefMessageToJson(message)
      tag.merge(metadata) { (_, new) in new }
      data["tag"] = tag
    }

    self.session?.invoke.resolve(data)
  }

  private func ndefMessageToJson(_ message: NFCNDEFMessage) -> JsonObject {
    var tag: JsonObject = [:]

    var records: [JsonObject] = []
    for record in message.records {
      var recordJson: JsonObject = [:]
      recordJson["tnf"] = record.typeNameFormat
      recordJson["type"] = record.type
      recordJson["id"] = record.identifier
      recordJson["payload"] = record.payload

      records.append(recordJson)
    }

    tag["records"] = records

    return tag
  }

  @objc public func scanNdef(_ invoke: Invoke) {
    DispatchQueue.main.async { [self] in
      self.startScanSession(invoke: invoke, kind: .ndef)
    }
  }

  @objc public func scanTag(_ invoke: Invoke) {
    DispatchQueue.main.async { [self] in
      self.startScanSession(invoke: invoke, kind: .tag)
    }
  }

  private func startScanSession(invoke: Invoke, kind: ScanKind) {
    let keepAlive = invoke.getBool("keepAlive", false)
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
        invalidateAfterFirstRead: true
      )
      break
    }

    nfcSession?.alertMessage = "Hold near NFC tag to scan."
    nfcSession?.begin()

    self.session = Session(
      nfcSession: nfcSession, invoke: invoke, keepAlive: keepAlive, tagProcessMode: .read)
  }
}

@_cdecl("init_plugin_nfc")
func initPlugin() -> Plugin {
  return NfcPlugin()
}
