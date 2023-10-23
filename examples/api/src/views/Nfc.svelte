<script>
  import { onMount, onDestroy } from "svelte";
  import { write, textRecord } from "@tauri-apps/plugin-nfc"

  export let onMessage;

  onMount(async () => {
    let record = textRecord("Hello from Tauri!", "someTestId")
    write([record]).then(onMessage).catch(onMessage)
  });
/*  onDestroy(() => {

  });*/
</script>

<div>
  WARNING: Writing mode enabled!
</div>
