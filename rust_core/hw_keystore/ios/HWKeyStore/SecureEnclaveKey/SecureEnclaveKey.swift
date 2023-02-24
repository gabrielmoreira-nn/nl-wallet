//
//  SecureEnclaveKey.swift
//  HWKeyStore
//
//  Created by Wallet Developer on 24/02/2023.
//

import Foundation
import Security

final class SecureEnclaveKey {
    // MARK: - Static properties

    // We want to return a key in PKIX, ASN.1 DER form, but SecKeyCopyExternalRepresentation
    // returns the coordinates X and Y of the public key as follows: 04 || X || Y. We convert
    // that to a valid PKIX key by prepending the SPKI of secp256r1 in DER format.
    // Based on https://stackoverflow.com/a/45188232
    private static let secp256r1Header = Data([
        0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01,
        0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00
    ])

    // MARK: - Static methods

    private static func tag(from identifier: String) -> Data {
        return identifier.data(using: .utf8)!
    }

    private static func throwFatalError(from unmanagedError: Unmanaged<CFError>?, message: String) -> Never {
        guard let unmanagedError else {
            fatalError(message)
        }

        let error = unmanagedError.takeRetainedValue() as Error

        fatalError("\(message): \(error)")
    }

    private static func fetchKey(with identifier: String) -> SecKey? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassKey,
            kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave,
            kSecAttrApplicationTag as String: self.tag(from: identifier),
            kSecAttrKeyType as String: kSecAttrKeyTypeEC,
            kSecReturnRef as String: true
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)

        switch status {
        case errSecSuccess:
            break
        case errSecItemNotFound:
            return nil
        default:
            guard #available(iOS 11.3, *),
                  let errorMessage = SecCopyErrorMessageString(status, nil) else {
                fatalError("Error while retrieving key with tag \"\(identifier)\"")
            }

            fatalError("Error while retrieving key with tag \"\(identifier)\": \(errorMessage)")
        }

        return (item as! SecKey)
    }

    private static func createKey(with identifier: String) -> SecKey {
        var error: Unmanaged<CFError>?

        guard let access = SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
            .privateKeyUsage,
            &error
        ) else {
            self.throwFatalError(from: error, message: "Error while creating key access control")
        }

        let keyAttributes: [String: Any] = [
            kSecAttrIsPermanent as String: true,
            kSecAttrApplicationTag as String: self.tag(from: identifier),
            kSecAttrAccessControl as String: access
        ]
        let attributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
            kSecAttrKeySizeInBits as String: 256,
            kSecAttrTokenID as String: kSecAttrTokenIDSecureEnclave,
            kSecPrivateKeyAttrs as String: keyAttributes
        ]

        guard let key = SecKeyCreateRandomKey(attributes as CFDictionary, &error) else {
            self.throwFatalError(from: error, message: "Error while creating private key")
        }

        return key
    }

    private static func derivePublicKey(from privateKey: SecKey) -> Data {
        guard let publicKey = SecKeyCopyPublicKey(privateKey) else {
            fatalError("Error while deriving public key")
        }

        var error: Unmanaged<CFError>?
        guard let keyData = SecKeyCopyExternalRepresentation(publicKey, &error) else {
            self.throwFatalError(from: error, message: "Error while encoding public key")
        }

        return self.secp256r1Header + (keyData as Data)
    }

    private static func sign(payload: Data, with privateKey: SecKey) -> Data {
        var error: Unmanaged<CFError>?
        guard let signature = SecKeyCreateSignature(privateKey,
                                                    .ecdsaSignatureRFC4754,
                                                    payload as CFData,
                                                    &error) else {
            self.throwFatalError(from: error, message: "Error while signing data")
        }

        return signature as Data
    }

    // MARK: - Instance properties

    let identifier: String
    private let privateKey: SecKey

    private(set) lazy var publicKey = Self.derivePublicKey(from: self.privateKey)

    // MARK: - Initializer

    init(identifier: String) {
        self.identifier = identifier

        self.privateKey = {
            guard let privateKey = Self.fetchKey(with: identifier) else {
                return Self.createKey(with: identifier)
            }

            return privateKey
        }()
    }

    // MARK: - Instance methods

    func sign(payload: Data) -> Data {
        return Self.sign(payload: payload, with: self.privateKey)
    }
}
