<script lang="ts">
  import anime from 'animejs';

  import Graphic from 'graphics/art-1.svg';
  import { onMount } from 'svelte';

  export let loopingAnimation = true;
  export let rotationAnimation = true;

  export let id = 'js-dashboard-graphic';

  onMount(() => {
    loopingAnimation &&
      anime({
        targets: `#${id} > g:not(.star)`,
        rotateY: function (_, i) {
          return i % 2 === 0 ? [0, 180] : [0, 90];
        },
        opacity: function (_, i) {
          return i % 2 === 0 ? 1 : [1, 0];
        },
        loop: true,
        direction: 'alternate',
        delay: (_, i) => 5000 + 150 * (i + 1),
        duration: 2500,
      });

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
    & :global(svg *) {
      will-change: transform, opacity;
      backface-visibility: hidden;
    }

    & :global(g) {
      transform-origin: 50% 50%;
    }

    & :global(.star) {
      transform-origin: 80px 80px;
    }
  }
</style>
