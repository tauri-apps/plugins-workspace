import Foundation

let BASE_URL = "https://api.revenuecat.com/v2"

func fetchProjectOverviewData(projectId: String, apiKey: String) async throws -> Data {
    let urlString = BASE_URL + "/projects/\(projectId)/metrics/overview"
    guard let url = URL(string: urlString) else {
        throw NSError(domain: "Invalid URL", code: -1)
    }
    
    var request = URLRequest(url: url)
    request.httpMethod = "GET"
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    
    let (data, response) = try await URLSession.shared.data(for: request)
    
    guard let httpResponse = response as? HTTPURLResponse,
          (200...299).contains(httpResponse.statusCode) else {
        throw NSError(domain: "HTTP Error", code: (response as? HTTPURLResponse)?.statusCode ?? -1)
    }
    
    return data
}

func createCustomer(projectId: String, apiKey: String, customerId: String, attributes: [String: Any]) async throws -> Data {
    let urlString = BASE_URL + "/projects/\(projectId)/customers"
    guard let url = URL(string: urlString) else {
        throw NSError(domain: "Invalid URL", code: -1)
    }
    
    // Create the request body
    let requestBody: [String: Any] = [
        "id": customerId,
        "attributes": attributes
        ]
    ]
    
    // Convert body to JSON data
    let jsonData = try JSONSerialization.data(withJSONObject: requestBody)
    
    // Create and configure the request
    var request = URLRequest(url: url)
    request.httpMethod = "POST"
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    request.httpBody = jsonData
    
    let (data, response) = try await URLSession.shared.data(for: request)
    
    guard let httpResponse = response as? HTTPURLResponse,
          (200...299).contains(httpResponse.statusCode) else {
        throw NSError(domain: "HTTP Error", code: (response as? HTTPURLResponse)?.statusCode ?? -1)
    }
    
    return data
}

func getCustomersData(projectId: String, apiKey: String) async throws -> Data {
    let urlString = BASE_URL + "/projects/\(projectId)/customers"
    guard let url = URL(string: urlString) else {
        throw NSError(domain: "Invalid URL", code: -1)
    }
    
    // Create and configure the request
    var request = URLRequest(url: url)
    request.httpMethod = "GET"
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    
    let (data, response) = try await URLSession.shared.data(for: request)
    
    guard let httpResponse = response as? HTTPURLResponse,
          (200...299).contains(httpResponse.statusCode) else {
        throw NSError(domain: "HTTP Error", code: (response as? HTTPURLResponse)?.statusCode ?? -1)
    }
    
    return data
}
