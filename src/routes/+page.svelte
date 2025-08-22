<script lang="ts">
  import { pushToast } from "../lib/stores/toast.svelte";

  // import { invoke } from "@tauri-apps/api/core";
</script>

<main class="w-screen h-screen text-primary-content">
  <button
    class="btn mt-10"
    on:click={() =>
      pushToast({ content: "MD3 Works!", alert: "error", duration: 30000 })}
    >Test MD3</button
  >
</main>
