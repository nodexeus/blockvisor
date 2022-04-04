<script lang="ts">
  import anime from 'animejs';

  import Graphic from 'graphics/art-2.svg';
  import { onMount } from 'svelte';

  export let rotationAnimation = true;

  export let id = 'js-search-graphic';

  onMount(() => {
    rotationAnimation &&
      anime({
        targets: `#${id} > .star`,
        rotateZ: function (_, i) {
          return i % 2 === 0 ? [0, 90] : [0, -90];
        },
        loop: true,
        easing: 'linear',
        duration: 1500,
      });
  });
</script>

<figure>
  <Graphic {id} />
</figure>

<style>
  figure {
    & :global(.star) {
      transform-origin: 80px 60px;
    }
  }
</style>
