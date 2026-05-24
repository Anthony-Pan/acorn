import Foundation
import Security

public struct Keychain: Sendable {
    public let service: String

    public static let acorn = Keychain(service: "app.acorn.desktop")

    public init(service: String) {
        self.service = service
    }

    public func save(account: String, value: String) throws {
        guard let data = value.data(using: .utf8) else {
            throw ProviderError.keychain("Could not encode value as UTF-8.")
        }
        let baseQuery: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
        ]
        let updateQuery: [CFString: Any] = [kSecValueData: data]
        let status = SecItemUpdate(baseQuery as CFDictionary, updateQuery as CFDictionary)
        if status == errSecItemNotFound {
            var insert = baseQuery
            insert[kSecValueData] = data
            insert[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
            let addStatus = SecItemAdd(insert as CFDictionary, nil)
            try Self.checkStatus(addStatus, operation: "add")
        } else {
            try Self.checkStatus(status, operation: "update")
        }
    }

    public func read(account: String) throws -> String? {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        switch status {
        case errSecSuccess:
            guard let data = result as? Data, let value = String(data: data, encoding: .utf8) else {
                throw ProviderError.keychain("Could not decode stored value as UTF-8.")
            }
            return value
        case errSecItemNotFound:
            return nil
        default:
            throw ProviderError.keychain("Read failed with status \(status).")
        }
    }

    public func delete(account: String) throws {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
        ]
        let status = SecItemDelete(query as CFDictionary)
        if status != errSecSuccess && status != errSecItemNotFound {
            throw ProviderError.keychain("Delete failed with status \(status).")
        }
    }

    public func has(account: String) throws -> Bool {
        try read(account: account) != nil
    }

    private static func checkStatus(_ status: OSStatus, operation: String) throws {
        guard status == errSecSuccess else {
            throw ProviderError.keychain("\(operation) failed with status \(status).")
        }
    }
}
