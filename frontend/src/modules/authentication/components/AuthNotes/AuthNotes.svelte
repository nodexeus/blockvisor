<script lang="ts">
  import type { UiContainer } from '@ory/kratos-client';
  import { getMessage } from 'modules/authentication/util';

  export let ui: UiContainer;

  $: formInfo = ui.messages
    ? ui.messages.filter((m) => m.type === 'info').map((e) => getMessage(e))
    : [];
</script>

{#if Boolean(formInfo?.length)}
  <aside class="t-tiny">
    <ul class="u-list-reset auth-info">
      {#each formInfo as info}
        <li class="auth-info__item">{info}</li>
      {/each}
    </ul>
  </aside>
{/if}

<style>
  .auth-info {
    display: flex;
    flex-direction: column;
    gap: 12px;

    &__item {
      padding: 8px 12px;
      border: 1px solid currentColor;
      border-radius: 4px;
      color: var(--color-text-3);
    }
  }
</style>
