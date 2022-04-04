<script lang="ts">
  export let author = '';
  export let role = '';
</script>

<section class="quote-block" {...$$props}>
  <article class="container container--large grid grid-spacing--small-only">
    <div class="quote-block__content t-medium">
      <slot name="content" />

      {#if author}
        <figure class="quote-block__author t-small">
          <slot name="image" />
          <figcaption>
            <div>{author}</div>
            <div class="quote-block__role">{role}</div>
          </figcaption>
        </figure>
      {/if}
    </div>
  </article>
</section>

<style>
  .quote-block {
    background-color: theme(--color-secondary);
    color: theme(--color-foreground-secondary);
    --quote-spacing: clamp(120px, 15vh, 160px);
    padding-top: 80px;
    padding-bottom: 80px;
    background-size: 73px 73px;
    background-image: url('/assets/images/pattern-star.png');
    animation: moveBackgroundRight 2.5s linear infinite;

    @media (--screen-large) {
      padding-top: var(--quote-spacing);
      padding-bottom: var(--quote-spacing);
    }

    & :global(p + p) {
      margin-top: 24px;
    }

    &__role {
      color: theme(--color-text-1-o30);
    }

    &__author {
      max-width: 200px;
      display: grid;
      grid-template-columns: 40px auto;
      grid-gap: 12px;
      margin-top: clamp(40px, 7vh, 60px);
    }

    &__content {
      grid-column-start: 1;
      grid-column-end: 13;

      @media (--screen-large) {
        grid-column-start: 3;
        grid-column-end: 9;
      }
    }
  }

  @keyframes moveBackgroundRight {
    0% {
      background-position: 0 0;
    }
    100% {
      background-position: 73px 0;
    }
  }
</style>
