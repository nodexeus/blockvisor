<script lang="ts">
  import { fade } from 'svelte/transition';
  export let transition = { duration: 0 };
</script>

<section in:fade|local={transition} class="grid-spacing layout-single-column">
  <slot />
</section>

<style>
  .layout-single-column {
    @media (--screen-medium-max) {
      margin-top: 40px;
    }
  }
</style>
