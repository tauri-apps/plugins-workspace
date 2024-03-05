// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation

import SwiftRs
import Tauri
import UIKit
import WebKit


struct SaveStore: Codable {
    let store: String
    let cache: [String: JSON]
}

class StorePlugin: Plugin {
    @objc public func save(_ invoke: Invoke) throws {
        do {
            let args = try invoke.parseArgs(SaveStore.self)
            let store = args.store
            let cache = args.cache
            let fileURL = getUrlFromPath(path: store, createDirs: true)
            
            try JSONEncoder().encode(cache).write(to: fileURL)
            invoke.resolve()
        } catch {
            invoke.reject(error.localizedDescription)
        }
    }
    
    @objc public func load(_ invoke: Invoke) throws {
        do {
            let path = try invoke.parseArgs(String.self)
            let fileURL = getUrlFromPath(path: path, createDirs: false)
            let data = try String(contentsOf: fileURL)
            let passData = dictionary(text: data)
            
            invoke.resolve(passData)
        } catch {
            invoke.reject(error.localizedDescription)
        }
    }
    
    func dictionary(text: String) -> [String: Any?] {
        if let data = text.data(using: .utf8) {
            do {
                return try JSONSerialization.jsonObject(with: data, options: []) as! [String: Any]
            } catch {
                fatalError(error.localizedDescription)
            }
        }
        
        return [:]
    }
    
    func getUrlFromPath(path: String, createDirs: Bool) -> URL {
        do {
            var url = try FileManager.default
                .url(
                    for: .applicationSupportDirectory,
                    in: .userDomainMask,
                    appropriateFor: nil,
                    create: true
                )
            let components = path.split(separator: "/").map { element in String(element) }
            
            if components.count == 1 {
                return url.appendPath(path: path, isDirectory: false)
            }
            
            for i in 0..<components.count {
                url = url.appendPath(path: components[i], isDirectory: true)
            }
            
            if components.count > 1 && createDirs {
                try FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
            }
            
            url = url.appendPath(path: components.last!, isDirectory: false)
            
            return url
        } catch {
            fatalError(error.localizedDescription)
        }
    }
}


@_cdecl("init_plugin_store")
func initPlugin() -> Plugin {
    return StorePlugin()
}

private extension URL {
    func appendPath(path: String, isDirectory: Bool) -> URL {
        if #available(iOS 16.0, *) {
            return self.appending(path: path, directoryHint: isDirectory ? .isDirectory : .notDirectory)
        } else {
            return self.appendingPathComponent(path, isDirectory: isDirectory)
        }
    }
}

public enum JSON : Codable {
    case null
    case number(NSNumber)
    case string(String)
    case array([JSON])
    case bool(Bool)
    case dictionary([String : JSON])
    
    public var value: Any? {
        switch self {
        case .null: return nil
        case .number(let number): return number
        case .string(let string): return string
        case .bool(let bool): return bool
        case .array(let array): return array.map { $0.value }
        case .dictionary(let dictionary): return dictionary.mapValues { $0.value }
        }
    }
    
    public init?(_ value: Any?) {
        guard let value = value else {
            self = .null
            return
        }
        
        if let bool = value as? Bool {
            self = .bool(bool)
        } else if let int = value as? Int {
            self = .number(NSNumber(value: int))
        } else if let double = value as? Double {
            self = .number(NSNumber(value: double))
        } else if let string = value as? String {
            self = .string(string)
        } else if let array = value as? [Any] {
            var mapped = [JSON]()
            for inner in array {
                guard let inner = JSON(inner) else {
                    return nil
                }
                
                mapped.append(inner)
            }
            
            self = .array(mapped)
        } else if let dictionary = value as? [String : Any] {
            var mapped = [String : JSON]()
            for (key, inner) in dictionary {
                guard let inner = JSON(inner) else {
                    return nil
                }
                
                mapped[key] = inner
            }
            
            self = .dictionary(mapped)
        } else {
            return nil
        }
    }
    
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        guard !container.decodeNil() else {
            self = .null
            return
        }
        
        if let bool = try container.decodeIfMatched(Bool.self) {
            self = .bool(bool)
        } else if let int = try container.decodeIfMatched(Int.self) {
            self = .number(NSNumber(value: int))
        } else if let double = try container.decodeIfMatched(Double.self) {
            self = .number(NSNumber(value: double))
        } else if let string = try container.decodeIfMatched(String.self) {
            self = .string(string)
        } else if let array = try container.decodeIfMatched([JSON].self) {
            self = .array(array)
        } else if let dictionary = try container.decodeIfMatched([String : JSON].self) {
            self = .dictionary(dictionary)
        } else {
            throw DecodingError.typeMismatch(JSON.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Unable to decode JSON as any of the possible types."))
        }
    }
    
    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        
        switch self {
        case .null: try container.encodeNil()
        case .bool(let bool): try container.encode(bool)
        case .number(let number):
            if number.objCType.pointee == 0x64 /* 'd' */ {
                try container.encode(number.doubleValue)
            } else {
                try container.encode(number.intValue)
            }
        case .string(let string): try container.encode(string)
        case .array(let array): try container.encode(array)
        case .dictionary(let dictionary): try container.encode(dictionary)
        }
    }
}

fileprivate extension SingleValueDecodingContainer {
    func decodeIfMatched<T : Decodable>(_ type: T.Type) throws -> T? {
        do {
            return try self.decode(T.self)
        } catch DecodingError.typeMismatch {
            return nil
        }
    }
}
