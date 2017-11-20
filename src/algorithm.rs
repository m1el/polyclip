//! Core Martinez-Rueda-Feito algorithm

/// Edge structure between exactly two points
pub struct Edge {
    /// The polygon this edge references
    pub polygon: &Polygon,
    /// Index of the point at which the point begins
    pub left_index: usize,
    /// Index of the point that ends the edge
    pub right_index: usize,
}

pub enum EventType {
    Left,
    Right,
}

/// Indicates if the edge belongs to the subject or clipping polygon
pub enum PolygonType {
    Subject,
    Clipping,
}

pub enum EdgeType {
    In,
    Out,
}

pub struct SweepEvent {
    /// Point associated with the event
    pub p: Point2D,
    /// Event associated to the other endpoint of the edge
    pub other: &SweepEvent,
    /// Is the point the left endpoint of the edge (p or other->p)?
    pub left: bool,
    /// Polygon type
    pub pl: PolygonType,
    /// Inside-outside transition into the polygon
    ///
    /// Indicates if the edge determines an inside-outside
    /// transition into the polygon, to which the edge
    /// belongs, for a vertical semi-line that goes up and
    /// intersects the edge
    pub in_out: bool,
    /// Is the edge (p, other->p) inside the other polygon?
    /// inside: indicates if the edge is inside the other polygon.

    pub is_inside: bool,
    /// Used for overlapping edges
    pub edge_type: EdgeType,
}

pub struct EventPoint {
    pub event_type: EventType,
}

/// Function to set inside and in_out flags for a left endpoint
///
/// current: current SweepEvent
/// previous: immediate predecessor of SweepEvent
pub fn set_inside_flag_of_left_endpoint(current: &mut SweepEvent, previous: Option<&SweepEvent>) {
    match ple {
        None => {
            // If ple is None then current is the first event in S
            // and the flags can be trivially set to false.
            current.is_inside = current.in_out = false;
        },
        Some(ple) => {
            if (current as *const _ as *const usize) == (ple as *const _ as *const usize) {
                // both reference the same polygon
                current.is_inside = previous.is_inside;
                current.in_out    = !previous.in_out;
            } else {
                current.is_inside = !previous.in_out;
                current.in_out = !previous.inside;
            }
        }
    }
}

/// Core function to calculate the intersection
pub fn calculate_intersect() {
    /*
    Insert the endpoints of the edges of polygons into priority queue Q

    while !Q.is_empty() {
        let event = Q.top();
        Q.pop();

        if (event.lef_endpoint()) {
            // left endpoint
            let pos = S.insert(event);
            event.set_inside_other_polygon_flag(S.prev(pos));
            possible_inter(pos, S.next(pos));
            possible_inter(pos, S.prev(pos));
        } else {
            // right endpoint
            let pos = S.find(*event.other);
            let next = S.next(pos);
            let pref = S.prev(pos);
            if event.inside_other_polygon() {
                intersection.add(event.segment());
            } else {
                Union.add(event.segment());
            }
            S.erase(pos);
            possible_inter(prev, next);
        }
    }
    */
}

/*
We must hold a set C - initially empty - of chains of
connected edges and a set R that holds the result
polygons.

Every edge e that belongs to the solution must
be processed as follows:

- If e cannot be connected at any of the ends of any chain
of C, then a new chain, formed by e, is added to C.
- If e can be connected to only one chain c of C, then e is
added to c. If the first and last edges in c are connected,
then c holds a result polygon and it is moved to R.
- If e can be connected to two chains c1 and c2 of C, then
the edges of c2 and e are added to c1, and c2 is removed
from C. If the first and last edges in c1 are connected
then c1 is moved to R.
*/


/// Subdivide the edges of the polygons at their intersection points.
pub fn subdivide_edges() { }

/// Select those subdivided edges that lie inside the other
/// polygon—or that do not lie depending on the operation.
pub fn select_edges() { }

/// Join the edges selected in step 2 to form the result polygon.
pub fn join_edges() { }