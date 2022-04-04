<script lang="ts">
  import AnimatedGraphic from '../AnimatedGraphic/AnimatedGraphic.svelte';
</script>

<article class="column--empty">
  <AnimatedGraphic {...$$restProps} />

  <div class="column--empty__content">
    <div class="t-medium">
      <slot name="title" />
    </div>
    <div class="column--empty__description t-small">
      <slot name="description" />
    </div>
  </div>
</article>

<style>
  .column--empty {
    margin: 56px 0;
    display: flex;
    align-items: center;
    gap: 8px 28px;
    flex-direction: column;

    @media (--screen-smaller) {
      flex-direction: row;
    }

    & > :global(figure) {
      flex-basis: 160px;
      flex-shrink: 0;
      min-width: 160px;
    }

    &__description {
      margin-top: 8px;
      color: theme(--color-text-3);
    }
  }
</style>
