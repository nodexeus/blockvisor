<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';
  import BroadcastEvent from './BroadcastEvent.svelte';

  const form = useForm({
    interval: { initial: 'anytime' },
  });

  const eventTypes = [
    'Add Gateway',
    'Assert Location',
    'Consensus Group',
    'Payments',
    'Rewards',
    'Stake Validator',
    'Transfer Hotspot',
    'Trasnfer Validator Stake',
    'Unstake Validator',
    "Price Oracle (doesn't require address)",
    "Chain Vars (doesn't require address)",
  ];
</script>

<form
  use:form
  class="add-broadcast"
  on:submit={(e) => {
    e.preventDefault();
  }}
>
  <ul class="u-list-reset add-broadcast__list">
    <li class="add-broadcast__item">
      <Input
        name="name"
        size="large"
        value={$form?.name?.value}
        field={$form?.name}
        validate={[required]}
        placeholder="Name your broadcast"
        required
      >
        <svelte:fragment slot="label">Broadcast Name</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Watch Address</div>

      <Input
        name="addresses (comma separated)"
        size="medium"
        value={$form?.address?.value}
        field={$form?.address}
        description="One or more wallet, hotspot, or validator addresses"
      >
        <svelte:fragment slot="label">Address</svelte:fragment>
      </Input>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Callback URL</div>

      <div class="add-broadcast__input">
        <Input
          name="callback"
          size="medium"
          value={$form?.callback?.value}
          field={$form?.callback}
          validate={[required]}
          description="ex: POST https://api.myproject.com/helium/events"
          required
        >
          <svelte:fragment slot="label">Callback URL</svelte:fragment>
          <svelte:fragment slot="hints">
            <Hint on="required">This is a mandatory field</Hint>
          </svelte:fragment>
        </Input>
      </div>

      <div class="add-broadcast__input">
        <Input
          name="callback"
          size="medium"
          value={$form?.callback?.value}
          field={$form?.callback}
          validate={[required]}
          description="Authorization: Bearer <Auth Token>"
          required
        >
          <svelte:fragment slot="label">Auth Token</svelte:fragment>
        </Input>
      </div>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Match these Events</div>

      {#each eventTypes as item}
        <BroadcastEvent name={item} value={item} />
      {/each}
    </li>
  </ul>

  <Button size="medium" style="secondary" type="submit">Add Broadcast</Button>
</form>

<style>
  .add-broadcast {
    margin-top: 60px;
  }

  .add-broadcast__checkbox + .add-broadcast__checkbox {
    margin-top: 16px;
  }

  .add-broadcast__label {
    margin-bottom: 24px;
  }

  .add-broadcast__list {
    margin-bottom: 44px;
  }

  .add-broadcast__item + :global(.add-broadcast__item) {
    border-top: 1px solid theme(--color-text-5-o10);
    margin-top: 44px;
    padding-top: 20px;
  }

  .add-broadcast__input {
    & + & {
      margin-top: 24px;
    }
  }
</style>
