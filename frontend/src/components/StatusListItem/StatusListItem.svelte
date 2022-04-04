<script lang="ts">
  export let content = '';
  export let status = '';
</script>

<li class="status-list-item">
  <figure class="status-list-item__figure">
    <slot />
    <figcaption class="t-xxlarge-fluid">{content}</figcaption>
  </figure>
  <p class="t-tiny status-list-item__status">{status}</p>
</li>

<style>
  .status-list-item {
    display: flex;
    align-items: baseline;
    gap: 12px;

    &__figure {
      display: flex;
      align-items: center;
      gap: 16px;
    }

    &__status {
      color: theme(--color-text-3);
    }

    & + :global(li) {
      margin-top: 20px;
    }
  }
</style>
