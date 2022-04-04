<script lang="ts">
  export let id;
  export let name;

  export let options = [];
</script>

{#if Boolean(options.length)}
  <div class="button-selector" role="radiogroup" aria-labelledby={id}>
    <span class="t-uppercase t-microlabel button-selector__title" {id}>
      <slot />
    </span>
    <ul class="u-list-reset button-selector__list">
      {#each options as { value, label, checked = false } (value)}
        <li class="button-selector__item">
          <input
            class="visually-hidden button-selector__input"
            type="radio"
            id={value}
            {name}
            {value}
            {checked}
            on:click
          />
          <label class="button-selector__label" for={value}>{label}</label>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .button-selector {
    align-items: baseline;
    gap: 8px 4px;
    flex-wrap: wrap;
    display: flex;

    &__label {
      @mixin font smaller;
      font-weight: var(--font-weight-bold);
      opacity: 0.5;
      cursor: pointer;
      transition: opacity 0.18s var(--transition-easing-cubic);
      padding: 0 20px;
    }

    &__input:hover:not(:checked) + &__label {
      opacity: 0.75;
    }

    &__input:checked + &__label {
      opacity: 1;
    }

    &__input:focus + &__label {
      outline: 2px solid theme(--color-text-5-o10);
      border-radius: 4px;
    }

    &__title {
      display: block;
      color: theme(--color-text-3);
      flex-basis: 100%;

      @media (--screen-medium) {
        flex-basis: auto;
      }
    }

    &__list {
      display: flex;
      overflow: auto;
      white-space: nowrap;
    }
  }
</style>
