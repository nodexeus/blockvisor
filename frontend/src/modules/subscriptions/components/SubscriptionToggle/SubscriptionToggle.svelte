<script lang="ts">
  import Button from 'components/Button/Button.svelte';

  export let isActive;
</script>

<Button
  style={isActive ? 'primary' : 'outline'}
  on:click
  size="small"
  {...$$restProps}
>
  <slot />
</Button>
