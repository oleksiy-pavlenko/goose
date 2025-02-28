import React from 'react';
import { Button } from '../../../../../ui/button';

interface ProviderSetupActionsProps {
  onCancel: () => void;
}

/**
 * Renders the "Submit" and "Cancel" buttons at the bottom.
 * Notice we rely on the parent's `onSubmit` in the form, so we only handle Cancel here.
 */
export default function ProviderSetupActions({ onCancel }: ProviderSetupActionsProps) {
  return (
    <div className="mt-[8px] -ml-8 -mr-8 pt-8">
      {/* We rely on the <form> "onSubmit" for the actual Submit logic */}
      <Button
        type="submit"
        variant="ghost"
        className="w-full h-[60px] rounded-none border-t border-borderSubtle text-md hover:bg-bgSubtle text-textProminent font-regular"
      >
        Submit
      </Button>
      <Button
        type="button"
        variant="ghost"
        onClick={onCancel}
        className="w-full h-[60px] rounded-none border-t border-borderSubtle hover:text-textStandard text-textSubtle hover:bg-bgSubtle text-md font-regular"
      >
        Cancel
      </Button>
    </div>
  );
}
