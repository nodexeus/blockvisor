<script lang="ts">
  import type { UiContainer } from '@ory/kratos-client';

  import { getMessage } from 'modules/authentication/util';

  export let ui: UiContainer;

  $: formErrors = ui.messages
    ? ui.messages.filter((m) => m.type === 'error').map((e) => getMessage(e))
    : [];
</script>

{#if Boolean(formErrors?.length)}
  <aside class="t-tiny">
    <ul class="u-list-reset auth-error">
      {#each formErrors as error}
        <li class="auth-error__item">{error}</li>
      {/each}
    </ul>
  </aside>
{/if}

<style>
  .auth-error {
    display: flex;
    flex-direction: column;
    gap: 12px;

    &__item {
      padding: 8px 12px;
      border: 1px solid currentColor;
      border-radius: 4px;
      color: theme(--color-utility-warning);
    }
  }
</style>
