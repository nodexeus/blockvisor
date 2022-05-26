<script lang="ts">
  import { goto } from '$app/navigation';

  import { session } from '$app/stores';
  import axios from 'axios';

  import Button from 'components/Button/Button.svelte';
  import LoadingSpinner from 'components/Spinner/LoadingSpinner.svelte';
  import { ROUTES } from 'consts/routes';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { required, Hint, useForm, email, minLength } from 'svelte-use-form';
  import { saveUserinfo } from 'utils';

  const form = useForm();

  let activeType: 'password' | 'text' = 'password';

  let isSubmitting = false;

  const handleToggle = () => {
    activeType = activeType === 'password' ? 'text' : 'password';
  };

  const handleLogin = async () => {
    isSubmitting = true;
    const { email, password } = $form.values;

    try {
      const res = await axios.post(ROUTES.AUTH_LOGIN, { email, password });
      saveUserinfo({ ...res.data, verified: true });
      isSubmitting = false;
      goto(ROUTES.DASHBOARD);
    } catch (error) {}
  };
</script>

<form
  method="POST"
  action="#"
  on:submit|preventDefault={handleLogin}
  class="login-form"
  use:form
>
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
    <li class="s-bottom--medium">
      <PasswordField
        field={$form?.password}
        name="password"
        placeholder="Password"
        labelClass="visually-hidden"
        validate={[required]}
        type={activeType}
      >
        <PasswordToggle on:click={handleToggle} {activeType} slot="utilRight" />
        <svelte:fragment slot="label">Password</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
        </svelte:fragment>
      </PasswordField>
    </li>
  </ul>

  <Button size="medium" display="block" style="primary" type="submit">
    {#if isSubmitting}
      &nbsp;
      <LoadingSpinner size="button" id="js-form-submit" />
    {:else}
      Login
    {/if}</Button
  >
</form>

<style>
  .login-form {
    & :global(button) {
      position: relative;
    }
  }
</style>
