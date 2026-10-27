import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "accent" | "outline" | "ghost";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: ReactNode;
}

const base =
  "inline-flex items-center justify-center gap-2 rounded-md px-4 py-2 " +
  "font-display text-sm font-semibold transition-colors duration-150 " +
  "disabled:cursor-not-allowed disabled:opacity-50";

const variants: Record<Variant, string> = {
  accent: "bg-accent text-surface hover:bg-accent-hover",
  outline: "border-2 border-primary text-primary hover:bg-primary hover:text-primary-foreground",
  ghost: "text-ink-muted hover:text-ink hover:bg-surface-warm/40",
};

export function Button({ variant = "outline", icon, className = "", children, ...rest }: ButtonProps) {
  return (
    <button className={`${base} ${variants[variant]} ${className}`} {...rest}>
      {icon}
      {children}
    </button>
  );
}
