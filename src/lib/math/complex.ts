export class Complex {
  constructor(public r: number, public i: number) {}

  static from(r: number, i: number = 0) {
    return new Complex(r, i);
  }

  add(other: Complex | number): Complex {
    if (typeof other === 'number') return new Complex(this.r + other, this.i);
    return new Complex(this.r + other.r, this.i + other.i);
  }

  sub(other: Complex | number): Complex {
    if (typeof other === 'number') return new Complex(this.r - other, this.i);
    return new Complex(this.r - other.r, this.i - other.i);
  }

  mul(other: Complex | number): Complex {
    if (typeof other === 'number') return new Complex(this.r * other, this.i * other);
    return new Complex(
      this.r * other.r - this.i * other.i,
      this.r * other.i + this.i * other.r
    );
  }

  div(other: Complex | number): Complex {
    if (typeof other === 'number') return new Complex(this.r / other, this.i / other);
    const den = other.r * other.r + other.i * other.i;
    return new Complex(
      (this.r * other.r + this.i * other.i) / den,
      (this.i * other.r - this.r * other.i) / den
    );
  }

  pow(other: Complex | number): Complex {
    if (typeof other === 'number') {
      const r = Math.pow(this.r * this.r + this.i * this.i, other / 2);
      const theta = Math.atan2(this.i, this.r) * other;
      return new Complex(r * Math.cos(theta), r * Math.sin(theta));
    }
    // (a+bi)^(c+di) = exp((c+di) * ln(a+bi))
    const ln = this.ln();
    return ln.mul(other).exp();
  }

  ln(): Complex {
    return new Complex(
      0.5 * Math.log(this.r * this.r + this.i * this.i),
      Math.atan2(this.i, this.r)
    );
  }

  exp(): Complex {
    const expR = Math.exp(this.r);
    return new Complex(expR * Math.cos(this.i), expR * Math.sin(this.i));
  }

  abs(): number {
    return Math.sqrt(this.r * this.r + this.i * this.i);
  }

  arg(): number {
    return Math.atan2(this.i, this.r);
  }

  negate(): Complex {
    return new Complex(-this.r, -this.i);
  }
}
