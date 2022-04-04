<script lang="ts">
  import Logo from 'components/Logo/Logo.svelte';
</script>

<header class="settings-layout__header">
  <Logo />
</header>

<main class="container container--large">
  <slot />
</main>

<style>
  .settings-layout__header {
    border-bottom: 1px solid theme(--color-text-5-o10);
    padding: 20px;
    margin-bottom: 56px;
  }
</style>
