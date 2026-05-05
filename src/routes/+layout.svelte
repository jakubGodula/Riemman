<script lang="ts">
  import '../app.css';
  let { children } = $props();
</script>

{@render children()}

<style>
  :global(body) {
    margin: 0;
    overflow: hidden;
  }
</style>
