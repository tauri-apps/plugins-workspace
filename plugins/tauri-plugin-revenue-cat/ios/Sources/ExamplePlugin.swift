import Tauri
import UIKit
import WebKit

class PingArgs: Decodable {
  let projectId: String
  let apiKey: String
}

class CreateCustomerArgs: Decodable {
  let projectId: String
  let apiKey: String
  let customerId: String
  let email: String
}

class FetchProjectOverviewDataArgs: Decodable {
  let projectId: String
  let apiKey: String
}

class GetCustomersDataArgs: Decodable {
  let projectId: String
  let apiKey: String
}

class ExamplePlugin: Plugin {
  @objc public func ping(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(PingArgs.self)
    invoke.resolve(["value": args.value ?? ""])
  }

  @objc public func createCustomer(_ invoke: Invoke) {
    Task {
      do {
        let args = try invoke.parseArgs(CreateCustomerArgs.self)
        let projectId = args.projectId
        let apiKey = args.apiKey
        let customerId = args.customerId
        let email = args.email
        let data = try await createCustomer(projectId: projectId, apiKey: apiKey, customerId: customerId, email: email)
        invoke.resolve(["value": String(data: data, encoding: .utf8) ?? ""])
      } catch {
        invoke.reject(error.localizedDescription)
      }
    }
  }

  @objc public func fetchProjectOverviewData(_ invoke: Invoke) {
    Task {
      do {
        let args = try invoke.parseArgs(FetchProjectOverviewDataArgs.self)
        let projectId = args.projectId
        let apiKey = args.apiKey
        let data = try await fetchProjectOverviewData(projectId: projectId, apiKey: apiKey)
        invoke.resolve(["value": String(data: data, encoding: .utf8) ?? ""])
      } catch {
        invoke.reject(error.localizedDescription)
      }
    }
  }

  @objc public func getCustomersData(_ invoke: Invoke) {
    Task {
      do {
        let args = try invoke.parseArgs(GetCustomersDataArgs.self)
        let projectId = args.projectId
        let apiKey = args.apiKey
        let data = try await getCustomersData(projectId: projectId, apiKey: apiKey)
        invoke.resolve(["value": String(data: data, encoding: .utf8) ?? ""])
      } catch {
        invoke.reject(error.localizedDescription)
      }
    }
  }
}

@_cdecl("init_plugin_revenue_cat")
func initPlugin() -> Plugin {
  return ExamplePlugin()
}
