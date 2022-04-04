<script lang="ts">
  import anime from 'animejs';

  import AnimateOnEntry from 'components/AnimateOnEntry/AnimateOnEntry.svelte';

  export let headerSize = 6;
  export let contentSize = 8;
  export let id = 'js-content';

  const headerClass = ['column', `column--size-${headerSize}`].join(' ');
  const contentClass = ['column', `column--size-${contentSize}`].join(' ');

  const animeOptions = {
    targets: `#${id} > *`,
    opacity: 1,
    translateY: '0',
    easing: 'easeOutQuad',
    delay: anime.stagger(250),
    duration: 350,
  };
</script>

<AnimateOnEntry
  {id}
  {animeOptions}
  type="div"
  class="container content-column-grid container--large grid grid-spacing--small-only"
>
  <header style="opacity:1; transform:translateY(0);" class={headerClass}>
    <slot name="header" />
  </header>
  <article style="opacity:0; transform:translateY(15px);" class={contentClass}>
    <slot name="content" />
  </article>
</AnimateOnEntry>

<style>
  :global(.content-column-grid) {
    grid-gap: 32px;
  }

  .column {
    &--size-4 {
      grid-column: span 4;
    }
    &--size-6 {
      grid-column: span 6;
    }
  }
</style>
