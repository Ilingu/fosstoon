<script>
  let { children } = $props();
  import "../app.css";
  import ToastManager from "../components/ToastManager.svelte";
</script>

{@render children()}

<ToastManager />
