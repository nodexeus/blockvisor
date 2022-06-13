<script lang="ts">
  import SkipToContent from 'modules/app/components/SkipToContent/SkipToContent.svelte';

  import { fade } from 'svelte/transition';
  import type { LayoutTwoColumnProps } from './LayoutTwoColumn.svelte';
  export let transition: LayoutTwoColumnProps['transition'] = { duration: 0 };
</script>

<div class="layout-two-column">
  <aside class="layout-two-column__sidebar grid-spacing">
    <SkipToContent target="main-content">Skip to the next section</SkipToContent
    >
    <slot name="sidebar" />
  </aside>

  <section
    id="main-content"
    tabindex="0"
    in:fade|local={transition}
    class="layout-two-column__content"
  >
    <slot />
  </section>
</div>

<style>
  .layout-two-column {
    display: flex;
    gap: 28px 0;
    flex-grow: 1;

    @media (--screen-xlarge) {
      gap: 108px 0;
    }

    @media (--screen-medium-max) {
      margin-top: 40px;
    }

    &__sidebar {
      z-index: 1;
      border-right: 1px solid theme(--color-text-5-o10);
      background-color: theme(--color-text-1);
      flex: 0 1 240px;
      min-height: 100%;
      min-width: 140px;

      & :global(.skip-to-content:focus) {
        position: absolute;
        @mixin font tiny;
        padding: 8px 12px;
      }

      @media (--screen-large) {
        position: sticky;
        top: 0;
      }

      @media (--screen-medium-max) {
        display: none;
      }
    }

    &__content {
      flex-grow: 1;
      flex-shrink: 1;
      overflow-x: hidden;
      padding: 0 80px 80px 20px;

      @media (--screen-medium-large) {
        padding-left: 28px;
      }

      @media (--screen-xlarge) {
        padding-left: 108px;
      }
    }
  }
</style>
