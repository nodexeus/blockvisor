<script lang="ts">
  import anime from 'animejs';

  import IconStar from 'icons/star-24.svg';

  import AnimateOnEntry from 'components/AnimateOnEntry/AnimateOnEntry.svelte';

  export let links = [];
  export let title = '';

  let animeOptions = {
    targets: '#js-styled-link-list > li',
    opacity: 1,
    translateY: 0,
    easing: 'easeOutQuad',
    delay: anime.stagger(200),
    duration: 300,
  };
</script>

<section class="styled-link-list">
  <div class="container container--large grid grid-spacing--small-only">
    <article class="styled-link-list__container container--large">
      <h2 class="t-label t-uppercase s-bottom--large styled-link-list__title">
        {title}
      </h2>
      <AnimateOnEntry
        {animeOptions}
        id="js-styled-link-list"
        class="u-list-reset"
        type="ul"
      >
        {#each links as { text, href }}
          <li
            style="transform: translateY(20px);"
            class="t-huge-fluid styled-link-list__item s-bottom--medium"
          >
            <a
              class="styled-link-list__link"
              {href}
              target="_blank"
              rel="noopener noreferrer"
            >
              {text}
              <IconStar class="icon" />
            </a>
          </li>
        {/each}
      </AnimateOnEntry>
    </article>
  </div>
</section>

<style>
  .styled-link-list {
    overflow: hidden;
    background-color: theme(--color-tertiary);
    color: theme(--color-text-1);
    padding-top: 40px;
    padding-bottom: 40px;

    @media (--screen-medium) {
      padding-top: clamp(40px, 7vh, 80px);
      padding-bottom: clamp(120px, 15vh, 160px);
    }

    &__item {
      opacity: 0;
    }

    &__title {
      color: theme(--color-text-3);
    }

    &__container {
      @media (--screen-large) {
        grid-column-start: 2;
        grid-column-end: 11;
      }
    }

    &__link {
      align-items: center;
      column-gap: 12px;
      transition: color 0.18s var(--transition-easing-cubic);
      text-decoration: none;
      color: theme(--color-text-1-o20);

      @media (--screen-medium-small) {
        display: inline-flex;
      }

      & :global(.icon) {
        min-width: 48px;
        flex-shrink: 0;
        transition: opacity 0.28s var(--transition-easing-cubic),
          transform 0.28s var(--transition-easing-cubic);
        opacity: 0;
        transform: translate3d(50%, 0, 0);
        display: none;

        @media (--screen-medium-small) {
          display: block;
        }
      }

      &:hover,
      &:active,
      &:focus {
        color: theme(--color-text-1);

        & :global(.icon) {
          opacity: 1;
          transform: translate3d(0, 0, 0);
        }
      }
    }
  }
</style>
