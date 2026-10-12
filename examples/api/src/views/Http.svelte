<script>
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
  import { invoke } from '@tauri-apps/api/core'
  import { JsonView } from '@zerodevx/svelte-json-view'

  let httpMethod = 'GET'
  let httpBody = ''

  export let onMessage

  async function makeHttpRequest() {
    let method = httpMethod || 'GET'

    const options = {
      method: method || 'GET',
      headers: {}
    }

    let bodyType

    if (method !== 'GET') {
      options.body = httpBody

      if (
        (httpBody.startsWith('{') && httpBody.endsWith('}')) ||
        (httpBody.startsWith('[') && httpBody.endsWith(']'))
      ) {
        options.headers['Content-Type'] = 'application/json'
        bodyType = 'json'
      } else if (httpBody !== '') {
        bodyType = 'text'
      }
    }

    const response = await tauriFetch('http://localhost:3003', options)
    const body =
      bodyType === 'json' ? await response.json() : await response.text()

    onMessage({
      url: response.url,
      status: response.status,
      ok: response.ok,
      headers: Object.fromEntries(response.headers.entries()),
      body
    })
  }

  /// http form
  let foo = 'baz'
  let bar = 'qux'
  let result = null

  async function doPost() {
    const form = new FormData()
    form.append('foo', foo)
    form.append('bar', bar)
    const response = await tauriFetch('http://localhost:3003/tauri', {
      method: 'POST',
      body: form
    })
    result = {
      url: response.url,
      status: response.status,
      ok: response.ok,
      headers: Object.fromEntries(response.headers.entries()),
      body: await response.text()
    }
  }

  /// request identity
  // the `userAgent` and `requestHeaders` rules in tauri.conf.json and the
  // `on_request` hook in src-tauri/src/cmd.rs apply to these paths; the echo
  // server mirrors the headers it received
  const identityPaths = [
    { path: '/plain', label: 'default user agent and origin' },
    { path: '/identity/rule', label: 'rule (replace mode)' },
    { path: '/identity/override/x', label: 'rule (override mode)' },
    { path: '/identity/no-origin/x', label: 'rule removing Origin' },
    { path: '/identity/hook/x', label: 'on_request hook' }
  ]
  const identityHeaders = [
    'user-agent',
    'origin',
    'x-identity',
    'x-hook-webview-label',
    'x-hook-webview-origin'
  ]
  let identityPath = identityPaths[0].path
  let frontendUserAgent = ''
  let frontendIdentity = ''
  let identityResult = null

  async function checkIdentity() {
    const headers = {}
    if (frontendUserAgent) headers['User-Agent'] = frontendUserAgent
    if (frontendIdentity) headers['X-Identity'] = frontendIdentity
    try {
      const response = await tauriFetch(`http://localhost:3003${identityPath}`, {
        headers
      })
      await response.text()
      identityResult = Object.fromEntries(
        identityHeaders.map((name) => [name, response.headers.get(name)])
      )
    } catch (e) {
      onMessage(e)
    }
  }

  /// cookie jar shared with Rust
  let cookieResult = null

  async function webviewCookieRequest() {
    try {
      const response = await tauriFetch('http://localhost:3003/cookies')
      await response.text()
      cookieResult = {
        source: 'webview',
        sent: response.headers.get('cookie'),
        set: response.headers.get('set-cookie')
      }
    } catch (e) {
      onMessage(e)
    }
  }

  async function nativeCookieRequest() {
    try {
      cookieResult = {
        source: 'rust',
        sent: await invoke('http_native_cookie')
      }
    } catch (e) {
      onMessage(e)
    }
  }

  async function clearCookies() {
    try {
      await invoke('http_clear_cookies')
      cookieResult = { cleared: true }
    } catch (e) {
      onMessage(e)
    }
  }
</script>

<form on:submit|preventDefault={makeHttpRequest}>
  <select class="input" id="request-method" bind:value={httpMethod}>
    <option value="GET">GET</option>
    <option value="POST">POST</option>
    <option value="PUT">PUT</option>
    <option value="PATCH">PATCH</option>
    <option value="DELETE">DELETE</option>
  </select>
  <br />
  <textarea
    class="input h-auto w-100%"
    id="request-body"
    placeholder="Request body"
    rows="5"
    bind:value={httpBody}
  ></textarea>
  <br />
  <button class="btn" id="make-request"> Make request </button>
</form>

<br />

<h3>HTTP Form</h3>

<div class="flex gap-2 children:grow">
  <input class="input" bind:value={foo} />
  <input class="input" bind:value={bar} />
</div>
<br />
<br />
<button class="btn" type="button" on:click={doPost}> Post it</button>
<br />
<br />
<JsonView json={result} />

<br />

<h3>Request identity</h3>

<div class="flex flex-col gap-2">
  <select class="input" id="identity-path" bind:value={identityPath}>
    {#each identityPaths as { path, label }}
      <option value={path}>{path} ({label})</option>
    {/each}
  </select>
  <input
    class="input"
    id="identity-user-agent"
    placeholder="User-Agent set by the frontend (optional)"
    bind:value={frontendUserAgent}
  />
  <input
    class="input"
    id="identity-x-identity"
    placeholder="X-Identity set by the frontend (optional)"
    bind:value={frontendIdentity}
  />
  <button class="btn" type="button" id="identity-send" on:click={checkIdentity}>
    Send and show received headers
  </button>
</div>
<br />
<JsonView json={identityResult} />

<br />

<h3>Cookie jar shared with Rust</h3>

<div class="flex flex-wrap gap-2">
  <button class="btn" type="button" on:click={webviewCookieRequest}>
    Webview request
  </button>
  <button class="btn" type="button" on:click={nativeCookieRequest}>
    Rust reqwest request
  </button>
  <button class="btn" type="button" on:click={clearCookies}>
    Clear cookie jar
  </button>
</div>
<br />
<JsonView json={cookieResult} />
