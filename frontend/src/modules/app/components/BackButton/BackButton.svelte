<script>
  import { browser } from '$app/env';

  import IconBack from 'icons/arrow-left-12.svg';
  import Button from 'components/Button/Button.svelte';

  const handleClick = () => {
    if (!browser) {
      return;
    }
    history.back();
  };
</script>

{#if browser && Boolean(window.history.length)}
  <Button
    style="outline"
    size="small"
    title="Go back"
    on:click={handleClick}
    {...$$restProps}
  >
    <slot><IconBack /> Back</slot>
  </Button>
{/if}
