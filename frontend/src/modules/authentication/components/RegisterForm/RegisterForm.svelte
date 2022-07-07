<script lang="ts">
  import { goto } from '$app/navigation';
  import axios from 'axios';
  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { ENDPOINTS } from 'consts/endpoints';
  import { ROUTES } from 'consts/routes';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { required, Hint, useForm, email, minLength } from 'svelte-use-form';
  import {
    passwordMatchConfirm,
    passwordMatchPassword,
    saveUserinfo,
  } from 'utils';

  const form = useForm();

  let activeType: 'password' | 'text' = 'password';

  let isSubmitting = false;

  const handleSubmit = async () => {
    isSubmitting = true;
    const { email, password, confirmPassword } = $form.values;

    try {
      const res = await axios.post(
        ENDPOINTS.USERS.CREATE_USER_POST,
        { email, password, password_confirm: confirmPassword },
        {
          headers: {
            Accept: 'application/json',
            'Content-Type': 'application/json',
          },
        },
      );
      saveUserinfo({ ...res.data, verified: true });
      goto(ROUTES.DASHBOARD);
    } catch (error) {
      isSubmitting = false;
    }
  };

  const handleToggle = () => {
    activeType = activeType === 'password' ? 'text' : 'password';
  };
</script>

<form on:submit|preventDefault={handleSubmit} use:form class="register-form">
  <ul class="u-list-reset">
    <li class="s-bottom--medium-small">
      <Input
        name="email"
        value={$form?.email?.value}
        field={$form?.email}
        validate={[required, email]}
        placeholder="Email"
        labelClass="visually-hidden"
        required
      >
        <svelte:fragment slot="label">Email</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">Your e-mail address is required</Hint>
          <Hint hideWhenRequired on="email">Email format is not correct</Hint>
        </svelte:fragment>
      </Input>
    </li>
    <li class="s-bottom--medium-small">
      <PasswordField
        field={$form?.password}
        name="password"
        placeholder="Password"
        labelClass="visually-hidden"
        validate={[required, passwordMatchConfirm, minLength(8)]}
        type={activeType}
      >
        <PasswordToggle on:click={handleToggle} {activeType} slot="utilRight" />
        <svelte:fragment slot="label">Password</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint hideWhenRequired on="passwordMatch">Passwords do not match</Hint
          >
          <Hint hideWhenRequired hideWhen="passwordMatch" on="minLength"
            >Password should be at least 8 characters long</Hint
          >
        </svelte:fragment>
      </PasswordField>
    </li>

    <li class="s-bottom--medium">
      <PasswordField
        field={$form?.confirmPassword}
        name="confirmPassword"
        placeholder="Confirm Password"
        labelClass="visually-hidden"
        validate={[required, passwordMatchPassword, minLength(8)]}
        type={activeType}
      >
        <PasswordToggle on:click={handleToggle} {activeType} slot="utilRight" />
        <svelte:fragment slot="label">Confirm password</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint hideWhenRequired on="passwordMatch">Passwords do not match</Hint
          >
          <Hint hideWhenRequired hideWhen="passwordMatch" on="minLength"
            >Password should be at least 8 characters long</Hint
          >
        </svelte:fragment>
      </PasswordField>
    </li>
  </ul>

  <Button size="medium" display="block" style="primary" type="submit">
    {#if isSubmitting}
      &nbsp;
      <LoadingSpinner size="button" id="js-form-submit" />
    {:else}
      Create account
    {/if}
  </Button>
</form>

<style>
  .register-form {
    & :global(button) {
      position: relative;
    }
  }
</style>
