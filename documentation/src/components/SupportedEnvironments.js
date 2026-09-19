import React from "react";
import Admonition from "@theme/Admonition";

const SupportedEnvironments = () => {
  return (
    <Admonition type="info" title="Supported Environments">
      The pleum CLI currently works on <strong>macOS</strong> and <strong>Linux</strong> systems and supports both <strong>ARM</strong> and <strong>x86</strong> architectures. 
      On <strong>Windows</strong>, pleum CLI can run via WSL, and pleum Desktop is natively supported. If you'd like to request support for additional operating systems, please{" "}
      <a
        href="https://github.com/gachon-star-want/pleumcode/discussions/867"
        target="_blank"
        rel="noopener noreferrer"
      >
        vote on GitHub
      </a>.
    </Admonition>
  );
};

export default SupportedEnvironments;
