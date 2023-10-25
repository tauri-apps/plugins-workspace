<script>
  import { onMount, onDestroy } from "svelte";
  import { write, textRecord } from "@tauri-apps/plugin-nfc"
  import { invoke } from "@tauri-apps/api/primitives";

  export let onMessage;

  onMount(async () => {
    let record = textRecord("Hello from Tauri!", "someTestId")
    //write([record]).then(m => onMessage(`success: ${m}`)).catch(m => onMessage(`error: ${m}`))
    // @ts-ignore
    const res = await invoke("plugin:nfc|scan", {kind:'ndef'}).catch(onMessage)
    // @ts-ignore
    onMessage(res)
    // @ts-ignore
    if (res && res.records) onMessage(res.records)
  });
/*  onDestroy(() => {

  });*/
</script>

<div>
  WARNING: Writing mode enabled!
</div>
