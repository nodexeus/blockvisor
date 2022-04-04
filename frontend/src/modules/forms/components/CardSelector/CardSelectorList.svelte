<script lang="ts">
  export let form;

  export let id = 'card-group';
</script>

<form
  aria-labelledby={id}
  use:form
  on:submit
  class="card-selector-list"
  role="radiogroup"
>
  <div class="card-selector-list__label" {id}>
    <slot name="label" />
  </div>

  <div class="card-selector-list__controls">
    <slot />
  </div>
</form>

<style>
  .card-selector-list {
    &__controls {
      display: flex;
      flex-wrap: wrap;
      gap: 28px;

      & > :global(*) {
        flex-basis: calc(33% - 28px);
        min-width: 178px;
        max-width: 200px;
      }
    }

    &__label {
      color: theme(--color-text-4);
      margin-bottom: 20px;
    }
  }
</style>
