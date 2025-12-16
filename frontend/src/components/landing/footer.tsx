"use client";

export const Footer = () => {
  return (
    <footer className="py-8 px-6 lg:px-12 bg-[#070709] border-t border-[#1e1f25]/50">
      <div className="max-w-6xl mx-auto flex items-center justify-between">
        <span className="text-sm text-[#64727C]">
          © {new Date().getFullYear()} Premonition
        </span>
        <a
          href="https://github.com/0xAmaan/prediction-terminal"
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-[#64727C] hover:text-[#9AA4AD] transition-colors"
        >
          GitHub
        </a>
      </div>
    </footer>
  );
};
