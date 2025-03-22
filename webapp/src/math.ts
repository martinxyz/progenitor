export function average(l: number[] | Float32Array): number | null {
    if (l.length == 0) return null
    return l.reduce((acc, val) => acc + val, 0) / l.length
}

export function clamp(number: number, min: number, max: number) {
    return Math.max(min, Math.min(number, max))
}

export function sigmoid(x: number): number {
    return 1.0 / (1.0 + Math.exp(-x))
}
