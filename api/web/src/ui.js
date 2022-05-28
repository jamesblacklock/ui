	(async function(w, d, wasm) {
		let __stringHandler = null;

		const wasm_imports = {
			runtime: {
				__send_string(ptr, len) {
					let str = uiPriv.getStringFromWasm(ptr, len);
					if(__stringHandler == null) {
						return uiPriv.addToHeap(str);
					} else {
						__stringHandler(str);
					}
				},
				__send_bool(value) {
					return uiPriv.addToHeap(!!value);
				},
				__send_f32(value) {
					return uiPriv.addToHeap(value);
				},
				__new_array() {
					return uiPriv.addToHeap([]);
				},
				__array_push(ptr, vptr) {
					const arr = uiPriv.getHeapObject(ptr);
					if(arr?.constructor != Array) {
						throw new Error(`expected Array, found '${arr}'`)
					}
					arr.push(uiPriv.getHeapObject(vptr));
				},
				__console_log(ptr, len) {
					const message = uiPriv.getStringFromWasm(ptr, len);
					console.log(message);
				},
				__throw_error(ptr, len) {
					const message = uiPriv.getStringFromWasm(ptr, len);
					throw new Error(message);
				},
				__create_text_node(ptr, len) {
					const content = uiPriv.getStringFromWasm(ptr, len);
					const element = d.createTextNode(content);
					return uiPriv.addToHeap(element);
				},
				__create_element(ptr, len) {
					const tag = uiPriv.getStringFromWasm(ptr, len);
					const element = d.createElement(tag);
					return uiPriv.addToHeap(element);
				},
				__next_sibling(node) {
					return uiPriv.addToHeap(uiPriv.getHeapNode(node).nextSibling);
				},
				__insert_before(node, insert, reference) {
					uiPriv.getHeapNode(node).insertBefore(
						uiPriv.getHeapNode(insert),
						uiPriv.getHeapNode(reference, true));
				},
				__append_child(node, child) {
					uiPriv.getHeapNode(node).appendChild(uiPriv.getHeapNode(child));
				},
				__first_child(node) {
					return uiPriv.addToHeap(uiPriv.getHeapNode(node).firstChild);
				},
				__remove(node) {
					uiPriv.getHeapNode(node).remove();
				},
				__set_text_content(node, ptr, len) {
					uiPriv.getHeapNode(node).textContent = uiPriv.getStringFromWasm(ptr, len);
				},
				__set_style(node, pptr, plen, vptr, vlen) {
					const prop = uiPriv.getStringFromWasm(pptr, plen);
					const value = uiPriv.getStringFromWasm(vptr, vlen);
					uiPriv.getHeapNode(node).style[prop] = value;
				},
				__update_event_listener(node, eptr, len, cptr) {
					const event = uiPriv.getStringFromWasm(eptr, len);
					const key = `__${event}_${cptr}`;
					
					node = uiPriv.getHeapNode(node);
					if(node[key]) {
						node.removeEventListener(event, node[key]);
					}
					node[key] = () => uiPriv.__dispatch_bound_callback(cptr);
					node.addEventListener(event, node[key]);
				},
				__heap_object_as_bool(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == Boolean) {
						return Number(object)|0;
					}
					return -1|0;
				},
				__heap_object_stage_string(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == String) {
						return uiPriv.stageString(object)|0;
					}
					return -1|0;
				},
				__heap_object_load_string(dest) {
					uiPriv.putStagedStringIntoWasm(dest);
				},
				__heap_object_as_f32(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == Number) {
						return object;
					}
					return Number.NaN;
				},
				__heap_object_is_function(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					return object?.constructor == Function;
				},
				__heap_object_call_function(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					if(object?.constructor == Function) {
						object();
					}
				},
				__heap_object_is_array(ptr) {
					const object = uiPriv.getHeapObject(ptr);
					return object?.constructor == Array;
				},
				__heap_object_get_property(ptr, keyptr, keylen) {
					const object = uiPriv.getHeapObject(ptr);
					if(!(object instanceof Object)) {
						return 0;
					}
					let key = uiPriv.getStringFromWasm(keyptr, keylen);
					if(object[key] == null) {
						return 0;
					}
					return uiPriv.addToHeap(object[key]);
				},
				__heap_object_drop(ptr) {
					uiPriv.dropFromHeap(ptr);
				},
			},
		};

		TYPE_SANITIZERS = {
			Length(value) {
				if(value?.constructor === Number) {
					return `${value}px`;
				} else if(value?.constructor === String) {
					const re = /^(\d*\.\d+|\d+)(px|in|cm|mm)$/;
					if(value.match(re) != null) {
						return value;
					}
				}
				return null;
			},
			Brush(value) {
				throw new Error("unimplemented!");
			},
			Alignment(value) {
				throw new Error("unimplemented!");
			},
			Int(value) {
				if(value?.constructor === Number) {
					return value|0;
				} else if(value?.constructor === String) {
					return parseInt(value)|0;
				}
				return null;
			},
			Float(value) {
				value = parseFloat(value);
				if(value == null || Number.isNaN(value)) {
					return null
				}
				return value;
			},
			String(value) {
				if(value?.constructor === String) {
					return value;
				} else if(value?.constructor === Number) {
					return String(value);
				}
				return null;
			},
			Boolean(value) {
				return !!value;
			},
			Callback(value, ctx) {
				if(value?.constructor === Function) {
					return value.bind(ctx);
				}
				return null;
			},
		}

		class Iterable {
			constructor(component, exports, key, baseType) {
				const name = component.constructor.name
				this.component = component;
				this.getter = getComponentExport(name, `${name}__get_${key}`, exports);
				this._getIndex = getComponentExport(name, `${name}__get_index_${key}`, exports);
				this._setIndex = getComponentExport(name, `${name}__set_index_${key}`, exports);
				this.key = key;
				this.baseType = baseType;
			}
			getIndex(index) {
				return uiPriv.dropFromHeap(this._getIndex(this.component.ptr, index|0));
			}
			setIndex(index, value) {
				value = uiPriv.sanitize(value, this.baseType, this.component);
				if(value == null) {
					return false;
				}
				this._setIndex(this.component.ptr, index|0, uiPriv.addToHeap(value));
				this.component.triggerUpdate();
				return true;
			}
			toJSON() {
				return uiPriv.dropFromHeap(this.getter(this.component.ptr));
			}
		}

		const uiPriv = {
			Iterable,
			components: {},
			heap: [null, true, false],
			freeHeapIndices: [],
			wasm: WebAssembly.instantiateStreaming(fetch(wasm), wasm_imports),
			decoder: new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }),
			encoder: new TextEncoder('utf-8'),

			getStringFromWasm(ptr, len) {
				if(len == 0) {
					return "";
				}
				const buffer = new Uint8Array(this.memory.buffer).subarray(ptr, ptr + len);
				return this.decoder.decode(buffer);
			},
			stageString(string) {
				this.stagedString = this.encoder.encode(string);
				return this.stagedString.length;
			},
			putStagedStringIntoWasm(ptr) {
				const buffer = new Uint8Array(this.memory.buffer).subarray(ptr, ptr + this.stagedString.length);
				buffer.set(this.stagedString);
				delete this.stagedString;
			},
			addToHeap(item) {
				if(item == null) {
					return 0;
				} else if(item === true) {
					return 1;
				} else if(item === false) {
					return 2;
				}
				let ptr = this.freeHeapIndices.pop();
				if(ptr == null) {
					ptr = this.heap.length;
					this.heap.push(item);
				} else {
					this.heap[ptr] = item;
				}
				return ptr;
			},
			dropFromHeap(ptr) {
				let result = this.heap[ptr];
				if(ptr > 2) {
					delete this.heap[ptr];
					this.freeHeapIndices.push(ptr);
				}
				return result;
			},
			getHeapObject(ptr) {
				return this.heap[ptr];
			},
			getHeapNode(ptr, optional) {
				let node = this.heap[ptr];
				if(optional && node == null) {
					return null;
				} else if(!(node instanceof Node)) {
					throw new Error(`"${node}" is not a node`);
				}
				return node;
			},
			getHeapCallback(ptr) {
				let f = this.heap[ptr];
				if(!(f instanceof Function)) {
					throw new Error(`"${f}" is not a function`);
				}
				return f;
			},
			sanitizeProps(props, propsDef, ctx) {
				let sanitized = {};
				for(let key in propsDef) {
					if(props[key] == null) {
						continue;
					}
					result = this.sanitize(props[key], propsDef[key], ctx);
					if(result != null) {
						sanitized[key] = result;
					}
				}
				return sanitized;
			},
			sanitize(value, type, ctx) {
				if(type instanceof Array) {
					if(value instanceof Array) {
						return value
							.map(e => this.sanitize(e, type[0], ctx))
							.filter(e => e != null);
					} else if(value?.constructor == Number && this.sanitize(value, type[0], ctx) != null) {
						return value;
					} else {
						return null;
					}
				} else if(type instanceof Object) {
					throw new Error('unimplemented!');
				}
				return TYPE_SANITIZERS[type]?.(value, ctx);
			},
			getProperty(component, getter, name, type) {
				return this.dropFromHeap(getter(component.ptr));
			},
			setProperty(component, setter, value, type) {
				value = this.sanitize(value, type, component);
				if(value == null) {
					return;
				}
				setter(component.ptr, this.addToHeap(value));
				component.triggerUpdate();
			},
			iterable(component, exports, name, baseType) {
				return new Proxy(new Iterable(component, exports, name, baseType), {
					get(target, prop) {
						const index = parseInt(prop);
						if(index == Number(prop)) {
							return target.getIndex(index);
						} else {
							return target[prop];
						}
					},
					set(target, prop, value) {
						const index = parseInt(prop);
						if(index == Number(prop)) {
							value = uiPriv.sanitize(value, target.baseType, target.component);
							return target.setIndex(index, value);
						}
						return false;
					},
				})
			},
		}

		function getComponentExport(name, exportName, exports) {
			if(!(exports[exportName] instanceof Function)) {
				throw new Error(`"${name}" is not a valid component (missing export: "${exportName}")`);
			}
			return exports[exportName]
		}

		const UI = {
			async loadComponent(name) {
				if(uiPriv.components[name] != null) {
					return uiPriv.components[name];
				}

				const wasm = await uiPriv.wasm;

				const componentPriv = {
					__attach_to_element: getComponentExport(name, `${name}__attach_to_element`, wasm.instance.exports),
					__render_component:  getComponentExport(name, `${name}__render_component`, wasm.instance.exports),
					__update_component:  getComponentExport(name, `${name}__update_component`, wasm.instance.exports),
					__new_component:     getComponentExport(name, `${name}__new_component`, wasm.instance.exports),
					__get_props_json:    getComponentExport(name, `${name}__get_props_json`, wasm.instance.exports),
				};
				
				let propsDef;
				{
					let json;
					__stringHandler = str => { json = str };
					componentPriv.__get_props_json();
					__stringHandler = null;
					propsDef = JSON.parse(json);
				}

				const Class = eval(`(class ${name} { constructor(props) { this.__constructor(props) } })`);
				Class.prototype.__constructor = function(props) {
					props = uiPriv.sanitizeProps(props, propsDef, this);
					const ptr = componentPriv.__new_component(uiPriv.addToHeap(props));
					Object.defineProperty(this, 'ptr', { value: ptr });
				};
				Class.prototype.propsDef = propsDef;
				Class.prototype.toJSON = function() {
					const result = {};
					for(key in propsDef) {
						if(propsDef[key] == 'Callback') {
							continue;
						} else if(this[key] instanceof Object) {
							result[key] = this[key].toJSON();
						} else {
							result[key] = this[key];
						}
					}
					return result;
				}
				Class.prototype.attachToElement = function(element) {
					if(typeof element == 'string') {
						element = d.getElementById(element);
					}
					if(!(element instanceof HTMLElement)) {
						console.error('failed to attach component to DOM: invalid element received');
						return;
					}
					let eptr = uiPriv.addToHeap(element);
					componentPriv.__attach_to_element(this.ptr, eptr);
					this.render();
				};
				Class.prototype.render = function() {
					componentPriv.__update_component(this.ptr);
					componentPriv.__render_component(this.ptr);
				};
				Class.prototype.triggerUpdate = function() {
					cancelAnimationFrame(this.animationFrame);
					this.animationFrame = requestAnimationFrame(() => {
						delete this.animationFrame;
						this.render();
					});
				};
				
				for(let key in propsDef) {
					let get;
					if(propsDef[key] == 'Callback') {
						let getter = getComponentExport(name, `${name}__call_${key}`, wasm.instance.exports);
						get = function() {
							return () => getter(this.ptr);
						}
					} else {
						let getter = getComponentExport(name, `${name}__get_${key}`, wasm.instance.exports);
						if(propsDef[key] instanceof Array) {
							get = function() {
								return uiPriv.iterable(this, wasm.instance.exports, key, type[0]);
							}
						} else {
							get = function() {
								return uiPriv.getProperty(this, getter, key, type);
							}
						}
					}

					let setter = getComponentExport(name, `${name}__set_${key}`, wasm.instance.exports);
					let type = propsDef[key];
					Object.defineProperty(Class.prototype, key, {
						enumerable: true,
						get,
						set(value) {
							uiPriv.setProperty(this, setter, value, propsDef[key]);
						},
					});
				}

				uiPriv.components[name] = Class;
				return Class;
			}
		};
		w.UI = UI;
		w.__uiPriv = uiPriv;
		uiPriv.wasm.then(wasm => {
			uiPriv.memory = wasm.instance.exports.memory;
			uiPriv.__dispatch_bound_callback = wasm.instance.exports.__dispatch_bound_callback;
		});
	})