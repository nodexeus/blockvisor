<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import Select from 'modules/forms/components/Select/Select.svelte';
  import TagsField from 'modules/forms/components/TagsField/TagsField.svelte';
  import { required, Hint, useForm } from 'svelte-use-form';

  const form = useForm({
    interval: { initial: 'anytime' },
  });
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
      <div class="add-broadcast__label">Broadcast Address</div>

      <Input
        name="address"
        size="small"
        value={$form?.address?.value}
        field={$form?.address}
      >
        <svelte:fragment slot="label">Address</svelte:fragment>
      </Input>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Add Callback URL</div>

      <Input
        name="callback"
        size="medium"
        value={$form?.callback?.value}
        field={$form?.callback}
        validate={[required]}
        description="Ex. https://api.com/callback?code=1234"
        required
      >
        <svelte:fragment slot="label">Callback URL</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Add Event Types</div>
      <TagsField
        value={$form?.eventTypes?.value}
        size="large"
        field={$form?.eventTypes}
        name="eventTypes"
        labelClass="visually-hidden"
        placeholder="Add Event Types"
      >
        <svelte:fragment slot="label">Add Event Types</svelte:fragment>
      </TagsField>
    </li>

    <li class="add-broadcast__item">
      <div class="add-broadcast__label">Set Broadcast Interval</div>
      <Select
        items={[{ value: 'anytime', label: 'Whenever it happens' }]}
        size="medium"
        field={$form?.interval}
        name="interval"
        validate={[required]}
        required
        description="How often you want to get data to your email"
      >
        <svelte:fragment slot="label">Interval</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Select>
    </li>
  </ul>

  <Button size="medium" style="secondary" type="submit">Add Broadcast</Button>
</form>

<style>
  .add-broadcast {
    margin-top: 60px;

    &__label {
      margin-bottom: 24px;
    }

    &__list {
      margin-bottom: 44px;
    }

    &__item {
      & + :global(.add-broadcast__item) {
        border-top: 1px solid theme(--color-text-5-o10);
        margin-top: 44px;
        padding-top: 20px;
      }
    }
  }
</style>
