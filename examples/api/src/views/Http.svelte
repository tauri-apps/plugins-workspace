<script>
  import { fetch as tauriFetch } from '@tauri-apps/plugin-http'
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
