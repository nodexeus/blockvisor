<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Dropdown from 'components/Dropdown/Dropdown.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import { Hint, required, useForm } from 'svelte-use-form';
  import BroadcastEvent from './BroadcastEvent.svelte';
  import DropdownLinkList from 'components/Dropdown/DropdownList.svelte';
  import DropdownItem from 'components/Dropdown/DropdownItem.svelte';
  import ButtonWithDropdown from 'modules/app/components/ButtonWithDropdown/ButtonWithDropdown.svelte';

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
    'Price Oracle',
    'Chain Vars',
  ];

  function handleSubmit() {
    console.log($form.values);
  }
</script>

<form use:form class="add-broadcast" on:submit|preventDefault={handleSubmit}>
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
      <Dropdown isActive={true}>
        <DropdownLinkList>
          <li>
            <DropdownItem as="button">Profile</DropdownItem>
          </li>
          <li>
            <DropdownItem as="button">Billing</DropdownItem>
          </li>
          <li>
            <DropdownItem as="button">Settings</DropdownItem>
          </li>
        </DropdownLinkList>
      </Dropdown>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Watch Address</div>

      <Input
        name="addresses"
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
          name="token"
          size="medium"
          value={$form?.token?.value}
          field={$form?.token}
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

  <p class="t-note s-top--medium s-bottom--xlarge">
    Note: any transaction matching this will trigger the callback.
  </p>

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
    margin-bottom: 26px;
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
