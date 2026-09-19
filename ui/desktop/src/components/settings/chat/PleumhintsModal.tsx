import { useState, useEffect } from 'react';
import { Button } from '../../ui/button';
import { Check } from '../../icons';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../ui/dialog';
import { errorMessage } from '../../../utils/conversionUtils';
import { defineMessages, useIntl } from '../../../i18n';

const i18n = defineMessages({
  dialogTitle: {
    id: 'pleumhintsModal.dialogTitle',
    defaultMessage: 'Configure Project Hints (.pleumhints)',
  },
  dialogDescription: {
    id: 'pleumhintsModal.dialogDescription',
    defaultMessage:
      'Provide additional context about your project to improve communication with Pleum',
  },
  helpText1: {
    id: 'pleumhintsModal.helpText1',
    defaultMessage:
      '.pleumhints is a text file used to provide additional context about your project and improve the communication with Pleum.',
  },
  helpText2: {
    id: 'pleumhintsModal.helpText2',
    defaultMessage:
      "Please make sure {bold} extension is enabled in the extensions page. This extension is required to use .pleumhints. You'll need to restart your session for .pleumhints updates to take effect.",
  },
  helpText3: {
    id: 'pleumhintsModal.helpText3',
    defaultMessage: 'See {link} for more information.',
  },
  helpTextLink: {
    id: 'pleumhintsModal.helpTextLink',
    defaultMessage: 'using .pleumhints',
  },
  errorReading: {
    id: 'pleumhintsModal.errorReading',
    defaultMessage: 'Error reading .pleumhints file: {error}',
  },
  fileFound: {
    id: 'pleumhintsModal.fileFound',
    defaultMessage: '.pleumhints file found at: {filePath}',
  },
  fileCreating: {
    id: 'pleumhintsModal.fileCreating',
    defaultMessage: 'Creating new .pleumhints file at: {filePath}',
  },
  placeholder: {
    id: 'pleumhintsModal.placeholder',
    defaultMessage: 'Enter project hints here...',
  },
  savedSuccessfully: {
    id: 'pleumhintsModal.savedSuccessfully',
    defaultMessage: 'Saved successfully',
  },
  close: {
    id: 'pleumhintsModal.close',
    defaultMessage: 'Close',
  },
  saving: {
    id: 'pleumhintsModal.saving',
    defaultMessage: 'Saving...',
  },
  save: {
    id: 'pleumhintsModal.save',
    defaultMessage: 'Save',
  },
  failedToAccess: {
    id: 'pleumhintsModal.failedToAccess',
    defaultMessage: 'Failed to access .pleumhints file',
  },
  failedToSave: {
    id: 'pleumhintsModal.failedToSave',
    defaultMessage: 'Failed to save .pleumhints file',
  },
  developer: {
    id: 'pleumhintsModal.developer',
    defaultMessage: 'Developer',
  },
});

const HelpText = () => {
  const intl = useIntl();

  return (
    <div className="text-sm flex-col space-y-4 text-text-secondary">
      <p>{intl.formatMessage(i18n.helpText1)}</p>
      <p>
        {intl.formatMessage(i18n.helpText2, {
          bold: <span className="font-bold">{intl.formatMessage(i18n.developer)}</span>,
        })}
      </p>
      <p>
        {intl.formatMessage(i18n.helpText3, {
          link: (
            <Button
              variant="link"
              className="text-blue-500 hover:text-blue-600 p-0 h-auto"
              onClick={() =>
                window.open(
                  'https://docs.pleum.ai/docs/guides/using-pleumhints/',
                  '_blank'
                )
              }
            >
              {intl.formatMessage(i18n.helpTextLink)}
            </Button>
          ),
        })}
      </p>
    </div>
  );
};

const ErrorDisplay = ({ error }: { error: Error }) => {
  const intl = useIntl();

  return (
    <div className="text-sm text-text-secondary">
      <div className="text-red-600">
        {intl.formatMessage(i18n.errorReading, { error: errorMessage(error) })}
      </div>
    </div>
  );
};

const FileInfo = ({ filePath, found }: { filePath: string; found: boolean }) => {
  const intl = useIntl();

  return (
    <div className="text-sm font-medium mb-2">
      {found ? (
        <div className="text-green-600">
          <Check className="w-4 h-4 inline-block" />{' '}
          {intl.formatMessage(i18n.fileFound, { filePath })}
        </div>
      ) : (
        <div>{intl.formatMessage(i18n.fileCreating, { filePath })}</div>
      )}
    </div>
  );
};

interface PleumhintsModalProps {
  directory: string;
  setIsPleumhintsModalOpen: (isOpen: boolean) => void;
}

export const PleumhintsModal = ({ directory, setIsPleumhintsModalOpen }: PleumhintsModalProps) => {
  const intl = useIntl();
  const pleumhintsFilePath = `${directory}/.pleumhints`;
  const [pleumhintsFile, setPleumhintsFile] = useState<string>('');
  const [pleumhintsFileFound, setPleumhintsFileFound] = useState<boolean>(false);
  const [pleumhintsFileReadError, setPleumhintsFileReadError] = useState<string>('');
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    const fetchPleumhintsFile = async () => {
      try {
        const { file, error, found } = await window.electron.readPleumhints();
        setPleumhintsFile(file);
        setPleumhintsFileFound(found);
        setPleumhintsFileReadError(error ?? '');
      } catch (error) {
        console.error('Error fetching .pleumhints file:', error);
        setPleumhintsFileReadError(intl.formatMessage(i18n.failedToAccess));
      }
    };
    if (directory) fetchPleumhintsFile();
  }, [directory, intl]);

  const writeFile = async () => {
    setIsSaving(true);
    setSaveSuccess(false);
    try {
      const saved = await window.electron.writePleumhints(pleumhintsFile);
      if (!saved) {
        throw new Error('Unable to save .pleumhints');
      }
      setSaveSuccess(true);
      setPleumhintsFileFound(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    } catch (error) {
      console.error('Error writing .pleumhints file:', error);
      setPleumhintsFileReadError(intl.formatMessage(i18n.failedToSave));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <Dialog open={true} onOpenChange={(open) => setIsPleumhintsModalOpen(open)}>
      <DialogContent className="w-[80vw] max-w-[80vw] sm:max-w-[80vw] max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>{intl.formatMessage(i18n.dialogTitle)}</DialogTitle>
          <DialogDescription>{intl.formatMessage(i18n.dialogDescription)}</DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto space-y-4 pt-2 pb-4">
          <HelpText />

          <div>
            {pleumhintsFileReadError ? (
              <ErrorDisplay error={new Error(pleumhintsFileReadError)} />
            ) : (
              <div className="space-y-2">
                <FileInfo filePath={pleumhintsFilePath} found={pleumhintsFileFound} />
                <textarea
                  value={pleumhintsFile}
                  className="w-full h-80 border rounded-md p-2 text-sm resize-none bg-background-primary text-text-primary border-border-primary focus:outline-none focus:ring-2 focus:ring-blue-500"
                  onChange={(event) => setPleumhintsFile(event.target.value)}
                  placeholder={intl.formatMessage(i18n.placeholder)}
                />
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          {saveSuccess && (
            <span className="text-green-600 text-sm flex items-center gap-1 mr-auto">
              <Check className="w-4 h-4" />
              {intl.formatMessage(i18n.savedSuccessfully)}
            </span>
          )}
          <Button variant="outline" onClick={() => setIsPleumhintsModalOpen(false)}>
            {intl.formatMessage(i18n.close)}
          </Button>
          <Button onClick={writeFile} disabled={isSaving}>
            {isSaving ? intl.formatMessage(i18n.saving) : intl.formatMessage(i18n.save)}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
