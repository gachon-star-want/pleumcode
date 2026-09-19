import { useColorMode } from '@docusaurus/theme-common';

export const PleumLogo = (props: { className?: string }) => {
  const { colorMode } = useColorMode();
  
  const logoSrc = colorMode === 'dark' 
    ? 'img/pleum-logo-white.png' 
    : 'img/pleum-logo-black.png';
  
  const logoAlt = 'pleum logo';

  return (
    <img
      src={logoSrc}
      alt={logoAlt}
      className={props.className}
      style={{ height: 'auto', maxWidth: '100%' }}
    />
  );
};
