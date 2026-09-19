import React from 'react';
import { PanelLeft } from 'lucide-react';

interface PleumBuiltinInstallerProps {
  extensionName: string;
  description?: string;
}

const PleumBuiltinInstaller: React.FC<PleumBuiltinInstallerProps> = ({
  extensionName,
  description
}) => {
  return (
    <div className="pleum-builtin-installer">
      <ol>
        <li>Click the <PanelLeft className="inline" size={16} /> button in the top-left to open the sidebar</li>
        <li>Click <code>Extensions</code> in the sidebar</li>
        <li>Toggle <code>{extensionName}</code> on</li>
      </ol>
    </div>
  );
};

export default PleumBuiltinInstaller;
