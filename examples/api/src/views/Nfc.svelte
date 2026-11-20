<script>
  import { onMount } from "svelte";
  import { write, scan, textRecord, uriRecord } from "@tauri-apps/plugin-nfc";
  import * as os from "@tauri-apps/plugin-os";

  export let onMessage;
  const decoder = new TextDecoder();

  let kind = "tag";
  let writeToNfc = false;
  let text = "";
  let uri = "";

  let scheme = "";
  let host = "";
  let pathPrefix = "";
  let mimeType = "";

  let isAndroid;
  onMount(async () => {
    isAndroid = (await os.platform()) === "android";
  });

  async function _readNfc() {
    onMessage(`NFC scanning ${kind}`);

    const tagResponse = await scan(
      {
        type: kind,
        uri: {
          scheme: scheme || null,
          host: host || null,
          pathPrefix: pathPrefix || null,
        },
        mimeType: mimeType || null,
      },
      {
        keepSessionAlive: writeToNfc,
        message: "Hold your iPhone near an NFC tag",
        successMessage: "Tag successfully read",
      }
    );

    onMessage({
      id: decoder.decode(new Uint8Array(tagResponse.id)),
      kind: tagResponse.kind,
      records: tagResponse.records.map((record) => ({
        id: decoder.decode(new Uint8Array(record.id)),
        kind: decoder.decode(new Uint8Array(record.kind)),
        payload: decoder.decode(new Uint8Array(record.payload)),
        tnf: record.tnf,
      })),
    });

    if (writeToNfc) {
      const records = [];
      if (text) {
        records.push(textRecord(text, "tauriTextId"));
      }
      if (uri) {
        records.push(uriRecord(uri, "tauriUriId"));
      }
      await write(records, {
        successMessage: "Data written to tag",
      });
      onMessage("Wrote to tag");
    }
  }

  function readNfc() {
    _readNfc().catch(onMessage);
  }
</script>

<div>
  <div class="flex gap-2 children:grow items-center">
    <div>
      <input type="checkbox" id="nfc-write" bind:checked={writeToNfc} />
      <label for="nfc-write">Write data</label>
    </div>

    <select class="input" id="request-method" bind:value={kind}>
      <option value="tag">TAG</option>
      <option value="ndef">NDEF</option>
    </select>
  </div>

  {#if isAndroid}
    <div class="flex flex-col gap-2 children:grow">
      <p>Filters</p>
      <div class="flex gap-2">
        <input
          class="input"
          placeholder="Scheme"
          style="width: 33%"
          bind:value={scheme}
        />
        <input
          class="input"
          placeholder="Host"
          style="width: 33%"
          bind:value={host}
        />
        <input
          class="input"
          placeholder="Path prefix"
          style="width: 33%"
          bind:value={pathPrefix}
        />
      </div>
      <div class="flex gap-2">
        <input class="input" placeholder="Mime type" bind:value={mimeType} />
      </div>
    </div>
  {/if}

  <div class="flex flex-col gap-2 children:grow">
    <p>Write Records</p>
    <div class="flex">
      <input class="input" placeholder="Text record" bind:value={text} />
      <input class="input" placeholder="URI record" bind:value={uri} />
    </div>
  </div>

  <button class="btn" on:click={readNfc}>Scan</button>
</div>
