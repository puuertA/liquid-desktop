// Analytic damped spring. Retargeting preserves both position and velocity.
export function springSample(value, velocity, target, dt, frequency = 19, damping = .82) {
  dt = Math.max(0, Math.min(dt, .25));
  const x = value - target, decay = damping * frequency;
  const omega = frequency * Math.sqrt(1 - damping * damping);
  const envelope = Math.exp(-decay * dt), cosine = Math.cos(omega * dt), sine = Math.sin(omega * dt);
  const b = (velocity + decay * x) / omega;
  const displacement = envelope * (x * cosine + b * sine);
  return {value:target + displacement,velocity:envelope * (-decay * (x * cosine + b * sine) + omega * (-x * sine + b * cosine))};
}
