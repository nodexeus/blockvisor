<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import { createUser } from 'modules/authentication/services';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { required, Hint, useForm, email } from 'svelte-use-form';
  import { passwordMatchConfirm, passwordMatchPassword } from 'utils';

  const form = useForm();

  let activeType: 'password' | 'text' = 'password';

  const handleToggle = () => {
    activeType = activeType === 'password' ? 'text' : 'password';
  };

  const handleCallback = (data) => {
    console.log(data);
  };

  const handleFormSubmit = (e) => {
    e.preventDefault();
    console.log($form);
    createUser(
      {
        email: $form.email.value,
        password: $form.password.value,
        password_confirm: $form.confirmPassword.value,
      },
      handleCallback,
    );
  };
</script>

<form use:form on:submit={handleFormSubmit}>
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
          <Hint on="email">Email format is not correct</Hint>
        </svelte:fragment>
      </Input>
    </li>
    <li class="s-bottom--medium-small">
      <PasswordField
        field={$form?.password}
        name="password"
        placeholder="Password"
        labelClass="visually-hidden"
        validate={[required, passwordMatchConfirm]}
        type={activeType}
      >
        <PasswordToggle on:click={handleToggle} {activeType} slot="utilRight" />
        <svelte:fragment slot="label">Password</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="passwordMatch">Passwords do not match</Hint>
        </svelte:fragment>
      </PasswordField>
    </li>

    <li class="s-bottom--medium">
      <PasswordField
        field={$form?.confirmPassword}
        name="confirmPassword"
        placeholder="Confirm Password"
        labelClass="visually-hidden"
        validate={[required, passwordMatchPassword]}
        type={activeType}
      >
        <PasswordToggle on:click={handleToggle} {activeType} slot="utilRight" />
        <svelte:fragment slot="label">Confirm password</svelte:fragment>
        <svelte:fragment slot="hints">
          <Hint on="required">This is a mandatory field</Hint>
          <Hint on="passwordMatch">Passwords do not match</Hint>
        </svelte:fragment>
      </PasswordField>
    </li>
  </ul>

  <Button size="medium" display="block" style="primary" type="submit"
    >Create account</Button
  >
</form>
