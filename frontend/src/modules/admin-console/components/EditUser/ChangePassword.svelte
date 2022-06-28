<script lang="ts">
  import Button from 'components/Button/Button.svelte';
  import Modal from 'components/Modal/Modal.svelte';
  import PasswordField from 'modules/forms/components/PasswordField/PasswordField.svelte';
  import PasswordToggle from 'modules/forms/components/PasswordToggle/PasswordToggle.svelte';
  import { Hint, minLength, required, useForm } from 'svelte-use-form';
  import { passwordMatchConfirm, passwordMatchPassword } from 'utils';

  export let handleModalClose: VoidFunction;
  export let isModalOpen: boolean = false;
  export let loading: boolean = false;
  export let loadingCreate: boolean = false;

  const form = useForm();

  let activeType: 'password' | 'text' = 'password';

  let isSubmitting = false;

  const handleSubmit = () => {
    isSubmitting = true;
  };

  const handleToggle = () => {
    activeType = activeType === 'password' ? 'text' : 'password';
  };
</script>

<Modal id="new-org" {handleModalClose} isActive={isModalOpen} size="normal">
  <svelte:fragment slot="header">Change Password</svelte:fragment>
  <form on:submit={handleSubmit} method="post" use:form class="register-form">
    <ul class="u-list-reset">
      <li class="s-bottom--large">
        <PasswordField
          field={$form?.password}
          name="password"
          placeholder="Password"
          validate={[required, passwordMatchConfirm, minLength(8)]}
          type={activeType}
        >
          <PasswordToggle
            on:click={handleToggle}
            {activeType}
            slot="utilRight"
          />
          <svelte:fragment slot="label">Current Password</svelte:fragment>
          <svelte:fragment slot="hints">
            <Hint on="required">This is a mandatory field</Hint>
            <Hint hideWhenRequired on="passwordMatch"
              >Passwords do not match</Hint
            >
            <Hint hideWhenRequired hideWhen="passwordMatch" on="minLength"
              >Password should be at least 8 characters long</Hint
            >
          </svelte:fragment>
        </PasswordField>
      </li>
      <li class="s-bottom--medium-small">
        <PasswordField
          field={$form?.password}
          name="password"
          placeholder="Password"
          validate={[required, passwordMatchConfirm, minLength(8)]}
          type={activeType}
        >
          <PasswordToggle
            on:click={handleToggle}
            {activeType}
            slot="utilRight"
          />
          <svelte:fragment slot="label">New Password</svelte:fragment>
          <svelte:fragment slot="hints">
            <Hint on="required">This is a mandatory field</Hint>
            <Hint hideWhenRequired on="passwordMatch"
              >Passwords do not match</Hint
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
          validate={[required, passwordMatchPassword, minLength(8)]}
          type={activeType}
        >
          <PasswordToggle
            on:click={handleToggle}
            {activeType}
            slot="utilRight"
          />
          <svelte:fragment slot="label">Confirm new password</svelte:fragment>
          <svelte:fragment slot="hints">
            <Hint on="required">This is a mandatory field</Hint>
            <Hint hideWhenRequired on="passwordMatch"
              >Passwords do not match</Hint
            >
            <Hint hideWhenRequired hideWhen="passwordMatch" on="minLength"
              >Password should be at least 8 characters long</Hint
            >
          </svelte:fragment>
        </PasswordField>
      </li>
    </ul>
  </form>

  <div slot="footer">
    <Button on:click={handleSubmit} size="small" style="primary"
      >Change Password</Button
    >
  </div>
</Modal>
