<script lang="ts">
  import IconButton from 'components/IconButton/IconButton.svelte';
  import IconArrow from 'icons/arrow-left-12.svg';
  import type { SubLinksProps } from './SubLinks.svelte';

  export let changeSubNav: SubLinksProps['changeSubNav'];

  const handleChangeNav = () => {
    changeSubNav('');
  };
</script>

<div class="sublinks__action">
  <IconButton on:click={handleChangeNav} style="outline" size="tiny">
    <IconArrow />
  </IconButton>
</div>

<slot />

<style>
  .sublinks {
    &__action {
      margin-bottom: 20px;
    }
  }
</style>
