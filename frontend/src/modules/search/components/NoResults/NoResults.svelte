<script lang="ts">
  import AnimatedGraphic from '../AnimatedGraphic/AnimatedGraphic.svelte';
</script>

<article class="results--empty">
  <div class="t-uppercase t-microlabel results--empty__title">No results</div>
  <AnimatedGraphic {...$$restProps} />

  <div class="results--empty__content">
    <div class="results--empty__description t-center t-tiny">
      <slot>
        Try repeating your search with different terms for better results.
      </slot>
    </div>
  </div>
</article>

<style>
  .results--empty {
    display: flex;
    align-items: center;
    gap: 40px;
    flex-direction: column;

    &__title {
      color: theme(--color-text-3);
      align-self: flex-start;
    }

    &__description {
      margin-top: 8px;
      color: theme(--color-text-5);
      max-width: 238px;
    }
  }
</style>
