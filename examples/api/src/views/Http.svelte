<script>
  import { fetch as tauriFetch } from "@tauri-apps/plugin-http";
  import { JsonView } from "@zerodevx/svelte-json-view";

  let httpMethod = "GET";
  let httpBody = "";

  export let onMessage;

  async function makeHttpRequest() {
    let method = httpMethod || "GET";

    const options = {
      method: method || "GET",
      headers: {},
    };

    let bodyType;

    if (method !== "GET") {
      options.body = httpBody;

      if (
        (httpBody.startsWith("{") && httpBody.endsWith("}")) ||
        (httpBody.startsWith("[") && httpBody.endsWith("]"))
      ) {
        options.headers["Content-Type"] = "application/json";
        bodyType = "json";
      } else if (httpBody !== "") {
        bodyType = "text";
      }
    }

    const response = await tauriFetch("http://localhost:3003", options);
    const body =
      bodyType === "json" ? await response.json() : await response.text();
    onMessage({
      url: response.url,
      status: response.status,
      body,
    });
  }

  /// http form
  let foo = "baz";
  let bar = "qux";
  let result = null;
  let multipart = true;

  async function doPost() {
    const response = await tauriFetch("http://localhost:3003", {
      method: "POST",
      body: {
        foo,
        bar,
      },
      headers: multipart
        ? { "Content-Type": "multipart/form-data" }
        : undefined,
    });
    result = {
      url: response.url,
      status: response.status,
      headers: JSON.parse(JSON.stringify(response.headers)),
      body: await response.text(),
    };
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
  />
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
<label>
  <input type="checkbox" bind:checked={multipart} />
  Multipart
</label>
<br />
<br />
<button class="btn" type="button" on:click={doPost}> Post it</button>
<br />
<br />
<JsonView json={result} />
