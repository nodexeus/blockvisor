<script>
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import MultiSelect from 'modules/forms/components/MultiSelect/MultiSelect.svelte';
  import { email, Hint, required, useForm } from 'svelte-use-form';

  const form = useForm();
</script>

<form
  class="container--small info-form"
  on:submit={(e) => {
    e.preventDefault();
    console.log($form);
  }}
  use:form
>
  <ul class="u-list-reset">
    <li class="info-form__item">
      <Input
        size="large"
        validate={[required]}
        name="firstname"
        field={$form?.firstname}
      >
        <svelte:fragment slot="label">First Name</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li class="info-form__item">
      <Input
        size="large"
        validate={[required]}
        name="lastname"
        field={$form?.lastname}
      >
        <svelte:fragment slot="label">Last Name</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li class="info-form__item">
      <Input
        size="large"
        validate={[required, email]}
        name="email"
        field={$form?.email}
      >
        <svelte:fragment slot="label">E-mail</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="email" hideWhenRequired>Email is not valid</Hint>
        </svelte:fragment>
      </Input>
    </li>

    <li class="info-form__item">
      <Input size="large" name="organization" field={$form?.organization}>
        <svelte:fragment slot="label">Organization</svelte:fragment>
      </Input>
    </li>

    <li class="info-form__item">
      <MultiSelect validate={[required]} field={$form?.group} name="group">
        <svelte:fragment slot="label">Group</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </MultiSelect>
    </li>
  </ul>
  <div class="info-form__actions">
    <Button style="secondary" size="medium" type="submit">Save</Button>
    <Button style="ghost" size="medium" type="reset">Discard Changes</Button>
  </div>
</form>

<style>
  .info-form {
    margin-top: 60px;

    &__item {
      margin-bottom: 24px;
    }
    &__actions {
      display: flex;
      flex-direction: column;
      margin-top: 40px;
      display: flex;
      gap: 12px;

      @media (--screen-smaller) {
        flex-direction: row;
      }
    }
  }
</style>
