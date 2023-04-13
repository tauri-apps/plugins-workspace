<script>
  import MessageDialogs from "./lib/MessageDialogs.svelte";
  import FileDialogs from "./lib/FileDialogs.svelte";
</script>

<main class="container">
  <div class="row">
    <MessageDialogs />
  </div>
  <div class="row">
    <FileDialogs />
  </div>
</main>

<style>
  .container > .row {
    margin-top: 24px;
  }
</style>
