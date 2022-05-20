<script lang="ts">
  import { SvelteToast, toast } from '@zerodevx/svelte-toast';

  import axios from 'axios';

  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { user } from 'modules/authentication/store';
  import BroadcastEvent from 'modules/broadcasts/components/AddBroadcast/BroadcastEvent.svelte';
  import {
    blockchains,
    getAllBlockchains,
  } from 'modules/broadcasts/store/broadcastStore';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import Select from 'modules/forms/components/Select/Select.svelte';
  import { onMount } from 'svelte';
  import { Hint, required, useForm } from 'svelte-use-form';

  let orgId: string;
  let isSubmitting: boolean;

  const form = useForm({
    interval: { initial: 'anytime' },
  });

  const BroadcastEvents = [
    { id: 'add_gateway_v1', value: 'Add Gateway' },
    { id: 'assert_location_v2', value: 'Assert Location' },
    { id: 'concensus_group_failure_v1', value: 'Consensus Group Failure' },
    { id: 'concensus_group_v1', value: 'Consensus Group' },
    { id: 'payment_v2', value: 'Payments' },
    { id: 'rewards_v2', value: 'Rewards' },
    { id: 'stake_validator_v1', value: 'Stake Validator' },
    { id: 'transfer_hotspot_v1', value: 'Transfer Hotspot' },
    { id: 'transfer_validator_stake_v1', value: 'Transfer Validator Stake' },
    { id: 'unstake_validator_v1', value: 'Unstake Validator' },
    { id: 'validator_heartbeat_v1', value: 'Validator Heartbeat' },
  ];

  onMount(() => {
    getAllBlockchains();

    axios
      .get('/api/broadcast/getOrganisationId', { params: { id: $user.id } })
      .then((res) => {
        if (res.statusText === 'OK') {
          orgId = res.data;
        }
      });
  });

  async function handleSubmit() {
    isSubmitting = true;

    let txn_types = '';
    BroadcastEvents.forEach((item, i) => {
      if ($form?.[item.id].value === 'checked') {
        txn_types += `${item.id}, `;
      }
    });

    const res = await axios.post('/api/broadcast/createNewBroadcast', {
      org_id: orgId,
      name: $form.name?.value,
      blockchain_id: $form.blockchain?.value,
      callback_url: $form.callback_url?.value,
      auth_token: $form.auth_token?.value,
      txn_types: txn_types,
      is_active: $form.is_active?.value,
      addresses: $form.addresses?.value,
    });

    if (res.statusText === 'OK') {
      toast.push('Succesfully added');
    }

    isSubmitting = false;
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

      <Select
        labelClass="s-top--medium"
        items={$blockchains
          .filter(
            (chain) =>
              chain.status === 'production' && chain.supports_broadcast,
          )
          .map((item) => {
            return {
              label: item.name,
              value: item.id,
            };
          })}
        value={$form.blockchain?.value}
        name="blockchain"
        field={$form?.blockchain}
        style="outline"
        size="medium"
        label="Select a date"
      >
        <svelte:fragment slot="label">Group</svelte:fragment>
      </Select>
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
          name="callback_url"
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
          name="auth_token"
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

      {#each BroadcastEvents as item}
        <BroadcastEvent name={item.id} value={item?.value} />
      {/each}
    </li>
  </ul>

  <p class="t-note s-top--medium s-bottom--xlarge">
    Note: any transaction matching this will trigger the callback.
  </p>

  <Button size="medium" style="secondary" type="submit"
    >{#if isSubmitting}
      &nbsp;
      <LoadingSpinner size="button" id="js-form-submit" />
    {:else}
      Add Broadcast
    {/if}</Button
  >
</form>

<style>
  .add-broadcast {
    margin-top: 60px;

    & :global(button) {
      position: relative;
    }
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
