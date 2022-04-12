<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Input from 'modules/forms/components/Input/Input.svelte';
  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { required, Hint, useForm, email } from 'svelte-use-form';

  const form = useForm();

  let activeType: 'password' | 'text' = 'password';

  const handleToggle = () => {
    activeType = activeType === 'password' ? 'text' : 'password';
  };
</script>

<form
  use:form
  on:submit={(e) => {
    e.preventDefault();
  }}
>
  <ul class="u-list-reset">
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

  <Button size="medium" display="block" style="primary" type="submit"
    >Reset Password</Button
  >
</form>
