<script lang="ts">
  import anime from 'animejs';
  import { afterUpdate, onMount } from 'svelte';
  import IntersectionObserver from 'svelte-intersection-observer';
  import type { AnimateOnEntryProps } from './AnimateOnEntry.svelte';

  export let timeline = null;
  export let beforeAnimation = () => {};
  export let once = true;
  export let type = 'div';
  export let rootMargin = '0px';
  export let threshold = 0.5; /* Middle */
  export let animeOptions: AnimateOnEntryProps['animeOptions'] = {
    opacity: 1,
    translateY: '0px',
    easing: 'easeOutQuad',
    duration: 1000,
  };

  let element;
  let isInView = false;

  $: options = { targets: element, ...animeOptions };

  onMount(beforeAnimation);

  const animateIfInView = () => {
    if (!isInView) {
      return;
    }

    if (timeline) {
      anime.timeline().add(timeline);
      return;
    }

    anime(options);
  };

  afterUpdate(animateIfInView);
</script>

<IntersectionObserver
  {element}
  bind:rootMargin
  bind:threshold
  bind:once
  bind:intersecting={isInView}
>
  {#if type === 'section'}
    <section {...$$restProps} bind:this={element}>
      <slot />
    </section>
  {:else if type === 'article'}
    <article {...$$restProps} bind:this={element}>
      <slot />
    </article>
  {:else if type === 'figure'}
    <figure {...$$restProps} bind:this={element}>
      <slot />
    </figure>
  {:else if type === 'p'}
    <p {...$$restProps} bind:this={element}>
      <slot />
    </p>
  {:else if type === 'ul'}
    <ul {...$$restProps} bind:this={element}>
      <slot />
    </ul>
  {:else}
    <div {...$$restProps} bind:this={element}><slot /></div>
  {/if}
</IntersectionObserver>
