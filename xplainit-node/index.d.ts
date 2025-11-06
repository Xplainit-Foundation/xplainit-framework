/**
 * Xplainit - Natural Language Explanations for JavaScript/Node.js
 * 
 * This module provides runtime tracing and natural language explanations
 * for JavaScript code execution.
 */

/**
 * Event statistics
 */
export interface Statistics {
  /** Total number of captured events */
  total_events: number;
  /** Number of function call events */
  function_calls: number;
  /** Number of variable operation events */
  variable_operations: number;
  /** Number of error events */
  errors: number;
}

/**
 * Enable tracing with default configuration
 * @returns true if successfully enabled
 */
export function enable(): boolean;

/**
 * Disable tracing
 * @returns true if successfully disabled
 */
export function disable(): boolean;

/**
 * Check if tracing is currently enabled
 * @returns true if enabled, false otherwise
 */
export function isEnabled(): boolean;

/**
 * Get all captured events as JSON string
 * @returns JSON string containing array of events
 */
export function getEvents(): string;

/**
 * Clear all captured events
 * @returns true if successfully cleared
 */
export function clearEvents(): boolean;

/**
 * Get statistics about captured events
 * @returns Statistics object with event counts
 */
export function getStatistics(): Statistics;

/**
 * Xplainit tracer class for object-oriented usage
 */
export class Xplainit {
  /**
   * Create a new Xplainit tracer instance
   */
  constructor();
  
  /**
   * Enable tracing for this instance
   */
  enable(): void;
  
  /**
   * Disable tracing for this instance
   */
  disable(): void;
  
  /**
   * Get captured events as JSON string
   * @returns JSON string containing array of events
   */
  getEvents(): string;
}
