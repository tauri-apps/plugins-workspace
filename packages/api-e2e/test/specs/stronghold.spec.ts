// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  scratchDir
} from '../helpers/index.js'

// The example derives the snapshot key from the password with argon2. Each
// test works on its own snapshot file under the spec's scratch directory,
// which the fs plugin cleans up; the snapshot path is resolved inside the page.

const dir = scratchDir('stronghold')
const password = 'e2e-password'
const clientName = 'e2e-client'

/** An arbitrary, valid BIP39 mnemonic, so derived keys are deterministic. */
const mnemonic =
  'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'

describePlugin('stronghold', () => {
  before(async () => {
    await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      if (await api.fs.exists(dir, { baseDir })) {
        await api.fs.remove(dir, { baseDir, recursive: true })
      }
      await api.fs.mkdir(dir, { baseDir, recursive: true })
    }, dir)
  })

  after(async () => {
    await tauri(async (api, dir) => {
      const baseDir = api.fs.BaseDirectory.AppData
      if (await api.fs.exists(dir, { baseDir })) {
        await api.fs.remove(dir, { baseDir, recursive: true })
      }
    }, dir)
  })

  it('store records round-trip and persist in the snapshot', async () => {
    const result = await tauri(
      async (api, dir, password, clientName) => {
        const path = await api.path.join(
          await api.path.appDataDir(),
          dir,
          'store.stronghold'
        )
        const value = Array.from(new TextEncoder().encode('top secret'))

        const stronghold = await api.stronghold.Stronghold.load(path, password)
        const client = await stronghold.createClient(clientName)
        await client.getStore().insert('key', value)
        const sameSession = await client.getStore().get('key')
        await stronghold.save()
        await stronghold.unload()

        const reopened = await api.stronghold.Stronghold.load(path, password)
        const store = (await reopened.loadClient(clientName)).getStore()
        const persisted = await store.get('key')
        const missing = await store.get('missing')
        await reopened.unload()

        return {
          sameSession: sameSession && Array.from(sameSession),
          persisted: persisted && new TextDecoder().decode(persisted),
          missing,
          fileExists: await api.fs.exists(path)
        }
      },
      dir,
      password,
      clientName
    )
    expect(result.sameSession).toEqual(
      Array.from(new TextEncoder().encode('top secret'))
    )
    expect(result.persisted).toBe('top secret')
    expect(result.missing).toBeNull()
    expect(result.fileExists).toBe(true)
  })

  it('store.remove returns the removed value', async () => {
    const result = await tauri(
      async (api, dir, password, clientName) => {
        const path = await api.path.join(
          await api.path.appDataDir(),
          dir,
          'remove.stronghold'
        )
        const stronghold = await api.stronghold.Stronghold.load(path, password)
        const store = (await stronghold.createClient(clientName)).getStore()
        await store.insert('key', [1, 2, 3])
        const removed = await store.remove('key')
        const after = await store.get('key')
        await stronghold.unload()
        return { removed: removed && Array.from(removed), after }
      },
      dir,
      password,
      clientName
    )
    expect(result).toEqual({ removed: [1, 2, 3], after: null })
  })

  it('a snapshot cannot be opened with the wrong password', async () => {
    const path = await tauri(
      async (api, dir, password, clientName) => {
        const path = await api.path.join(
          await api.path.appDataDir(),
          dir,
          'password.stronghold'
        )
        const stronghold = await api.stronghold.Stronghold.load(path, password)
        await stronghold.createClient(clientName)
        await stronghold.unload()
        return path
      },
      dir,
      password,
      clientName
    )
    const error = await tauriError(
      (api, path) => api.stronghold.Stronghold.load(path, 'wrong-password'),
      path
    )
    expect(error).toMatch(/failed to decode\/decrypt/)
  })

  it('loading a client that was never created is rejected', async () => {
    const error = await tauriError(
      async (api, dir, password) => {
        const path = await api.path.join(
          await api.path.appDataDir(),
          dir,
          'no-client.stronghold'
        )
        const stronghold = await api.stronghold.Stronghold.load(path, password)
        try {
          await stronghold.loadClient('never-created')
        } finally {
          await stronghold.unload()
        }
      },
      dir,
      password
    )
    expect(error).toMatch(/error loading client data/)
  })

  it('vault procedures derive keys and sign without exposing secrets', async () => {
    const result = await tauri(
      async (api, dir, password, clientName, mnemonic) => {
        const { Location } = api.stronghold
        const path = await api.path.join(
          await api.path.appDataDir(),
          dir,
          'vault.stronghold'
        )
        const stronghold = await api.stronghold.Stronghold.load(path, password)
        const vault = (await stronghold.createClient(clientName)).getVault(
          'vault'
        )

        // the same mnemonic recovered twice derives the same key
        const seedA = Location.generic('vault', 'seed-a')
        const seedB = Location.generic('vault', 'seed-b')
        await vault.recoverBIP39(mnemonic, seedA)
        await vault.recoverBIP39(mnemonic, seedB)
        // Ed25519 SLIP-10 only derives hardened indices (the high bit set).
        // (No named helper: the transpiler would wrap it in a `__name` call
        // that does not exist in the page.)
        const chain = [44, 4218, 0, 0, 0].map((i) => (i | 0x80000000) >>> 0)
        const otherChain = [44, 4218, 0, 0, 1].map(
          (i) => (i | 0x80000000) >>> 0
        )
        const keyA = Location.generic('vault', 'key-a')
        const keyB = Location.generic('vault', 'key-b')
        const keyOther = Location.generic('vault', 'key-other')
        await vault.deriveSLIP10(chain, 'Seed', seedA, keyA)
        await vault.deriveSLIP10(chain, 'Seed', seedB, keyB)
        await vault.deriveSLIP10(otherChain, 'Seed', seedA, keyOther)

        const publicA = Array.from(await vault.getEd25519PublicKey(keyA))
        const publicB = Array.from(await vault.getEd25519PublicKey(keyB))
        const publicOther = Array.from(
          await vault.getEd25519PublicKey(keyOther)
        )
        const signature = Array.from(await vault.signEd25519(keyA, 'message'))
        const signatureAgain = Array.from(
          await vault.signEd25519(keyA, 'message')
        )

        // a random seed and a raw secret can be stored and removed
        const random = Location.generic('vault', 'random')
        await vault.generateSLIP10Seed(random)
        await vault.insert('raw', [9, 9, 9])
        await vault.remove(Location.generic('vault', 'raw'))

        // a generated mnemonic recovers the seed generateBIP39 stored
        const generatedSeed = Location.generic('vault', 'generated')
        const mnemonicBytes = await vault.generateBIP39(generatedSeed)
        const recoveredSeed = Location.generic('vault', 'recovered')
        await vault.recoverBIP39(
          new TextDecoder().decode(mnemonicBytes),
          recoveredSeed
        )
        const generatedKey = Location.generic('vault', 'generated-key')
        const recoveredKey = Location.generic('vault', 'recovered-key')
        await vault.deriveSLIP10(chain, 'Seed', generatedSeed, generatedKey)
        await vault.deriveSLIP10(chain, 'Seed', recoveredSeed, recoveredKey)
        const generatedPublic = Array.from(
          await vault.getEd25519PublicKey(generatedKey)
        )
        const recoveredPublic = Array.from(
          await vault.getEd25519PublicKey(recoveredKey)
        )

        await stronghold.unload()
        return {
          publicA,
          publicB,
          publicOther,
          signature,
          signatureAgain,
          generatedPublic,
          recoveredPublic
        }
      },
      dir,
      password,
      clientName,
      mnemonic
    )
    expect(result.publicA).toHaveLength(32)
    expect(result.publicB).toEqual(result.publicA)
    expect(result.publicOther).not.toEqual(result.publicA)
    // Ed25519 signatures are 64 bytes and deterministic
    expect(result.signature).toHaveLength(64)
    expect(result.signatureAgain).toEqual(result.signature)
    expect(result.recoveredPublic).toEqual(result.generatedPublic)
    expect(result.generatedPublic).not.toEqual(result.publicA)
  })
})
